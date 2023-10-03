use std::{cmp, fmt, mem};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, RwLock};
use byteorder::{LittleEndian, ReadBytesExt};
use vfs::{FileSystem, SeekAndRead, VfsFileType, VfsMetadata, VfsResult};
use vfs::error::VfsErrorKind;

const MAGIC_DECOMPRESSED: i32    = 0x00000000;
const MAGIC_COMPRESSED:   i32    = 0x43707273;
const MAGIC_ENCRYPTED:    i32    = 0x456e6372;
const MAGIC_VERSION:      i32    = 0x56657273;
const PROPERTY_PREFIX:    &str   = "prefix";
const PATH_SEPARATOR:     char   = '\\';
type  BankFsHandle               = Arc<RwLock<BankFsImpl>>;
type  EntryContent               = Vec<u8>;

pub struct BankFs {
    handle:               BankFsHandle
}

impl Debug for BankFs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("In Memory Bank File System")
    }
}

impl Default for BankFs {
    fn default() -> Self {
        Self::new()
    }
}

impl BankFs {
    pub fn new() -> Self {
        BankFs {
            handle: Arc::new(RwLock::new(BankFsImpl::new(PROPERTY_PREFIX))),
        }
    }

    fn ensure_has_parent(&self, path: &str) -> VfsResult<()> {
        let separator = path.rfind(PATH_SEPARATOR);
        if let Some(index) = separator {
            if self.exists(&path[..index])? {
                return Ok(());
            }
        }
        Err(VfsErrorKind::Other("Parent path does not exist".into()).into())
    }
}

impl FileSystem for BankFs {



    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item=String> + Send>> {
        let normalized_path = BankFsImpl::normalize_path(path, true);
        let handle = self.handle.read().unwrap();

        let mut found_folder = false;

        #[allow(clippy::needless_collect)]
        let entries: Vec<_> = handle
            .files
            .iter()
            .filter_map(|(candidate_path, _)| {
                if candidate_path == path {
                    found_folder = true;
                }
                if candidate_path.starts_with(&normalized_path) {
                    let rest = &candidate_path[normalized_path.len()..];
                    if !rest.contains(PATH_SEPARATOR) {
                        return Some(rest.to_string());
                    }
                }
                None
            })
            .collect();

        if !found_folder {
            return Err(VfsErrorKind::FileNotFound.into())
        }

        Ok(Box::new(entries.into_iter()))
    }

    fn create_dir(&self, path: &str) -> VfsResult<()> {
        let normalized_path = BankFsImpl::normalize_path(path, false); // leave trailing slash out
        self.ensure_has_parent(&*normalized_path)?;
        let map = &mut self.handle.write().unwrap().files;
        let entry = map.entry(path.to_string());

        match entry {
            Entry::Occupied(file) => {
                return match file.get().file_type {
                    VfsFileType::File => Err(VfsErrorKind::FileExists.into()),
                    VfsFileType::Directory => Err(VfsErrorKind::DirectoryExists.into()),
                }
            }
            Entry::Vacant(_) => {
                map.insert(
                    normalized_path,
                    BankEntry {
                        file_type: VfsFileType::Directory,
                        content: Default::default(),
                    },
                );
            }
        }

        Ok(())
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        let normalized_path = BankFsImpl::normalize_path(path, false);
        let handle = self.handle.read().unwrap();
        let file = handle.files.get(&*normalized_path).ok_or(VfsErrorKind::FileNotFound)?;
        ensure_file(file)?;
        Ok(Box::new(ReadableFile {
            content: file.content.clone(),
            position: 0,
        }))
    }


    fn create_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        let normalized_path = BankFsImpl::normalize_path(path, false);

        self.ensure_has_parent(&*normalized_path)?;
        let content = Arc::new(Vec::<u8>::new());
        self.handle.write().unwrap().files.insert(
            path.to_string(),
            BankEntry {
                file_type: VfsFileType::File,
                content,
            },
        );
        let writer = WritableFile {
            content: Cursor::new(vec![]),
            destination: path.to_string(),
            file_system: self.handle.clone(),
        };
        Ok(Box::new(writer))
    }

    fn append_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        let normalized_path = BankFsImpl::normalize_path(path, false);
        let handle = self.handle.write().unwrap();
        let file = handle.files.get(path).ok_or(VfsErrorKind::FileNotFound)?;
        let mut content = Cursor::new(file.content.as_ref().clone());
        content.seek(SeekFrom::End(0))?;
        let writer = WritableFile {
            content,
            destination: normalized_path,
            file_system: self.handle.clone(),
        };
        Ok(Box::new(writer))
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        let normalized_path = BankFsImpl::normalize_path(path, false);
        let guard = self.handle.read().unwrap();
        let files = &guard.files;
        let file = files.get(&*normalized_path).ok_or(VfsErrorKind::FileNotFound)?;
        Ok(VfsMetadata {
            file_type: file.file_type,
            len: file.content.len() as u64,
        })
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        let normalized_path = BankFsImpl::normalize_path(path, false);
        Ok(self.handle.read().unwrap().files.contains_key(&*normalized_path))
    }

    fn remove_file(&self, path: &str) -> VfsResult<()> {
        let mut handle = self.handle.write().unwrap();
        handle
            .files
            .remove(path)
            .ok_or(VfsErrorKind::FileNotFound)?;
        Ok(())
    }

    fn remove_dir(&self, path: &str) -> VfsResult<()> {
        if self.read_dir(path)?.next().is_some() {
            return Err(VfsErrorKind::Other("Directory to remove is not empty".into()).into());
        }
        let mut handle = self.handle.write().unwrap();
        handle
            .files
            .remove(path)
            .ok_or(VfsErrorKind::FileNotFound)?;
        Ok(())
    }
}
//--------------------------------------------------------------------------------------------------
#[derive()]
struct EntryInfo {
    filename:          String,
    mime:              i32,
    original_size:     i32,
    deprecated_offset: i32,
    timestamp:         i32,
    packed_size:       i32,
}

impl EntryInfo {
    #[inline]
    fn read_entry_meta(reader: &mut impl Read, max_name_length: usize) -> Result<EntryInfo, Box<dyn std::error::Error>> {
        let filename = read_utf8z(reader, max_name_length);
        let mime = reader.read_i32::<LittleEndian>()?;
        let original_size = reader.read_i32::<LittleEndian>()?;
        let deprecated_offset = reader.read_i32::<LittleEndian>()?;
        let timestamp = reader.read_i32::<LittleEndian>()?;
        let packed_size = reader.read_i32::<LittleEndian>()?;

        let entry = EntryInfo {
            filename,
            mime,
            original_size,
            deprecated_offset,
            timestamp,
            packed_size,
        };

        Ok(entry)
    }
    #[inline]
    fn blank(&self) -> bool {
        self.filename.is_empty() &&
        self.mime == MAGIC_DECOMPRESSED &&
        self.original_size == 0 &&
        self.deprecated_offset == 0 &&
        self.timestamp == 0 &&
        self.packed_size == 0
    }

    #[inline]
    fn version(&self) -> bool {
        self.mime == MAGIC_VERSION
    }

    #[inline]
    fn decompressed(&self) -> bool {
        self.mime == MAGIC_DECOMPRESSED
    }

    #[inline]
    fn encrypted(&self) -> bool {
        self.mime == MAGIC_ENCRYPTED
    }

    #[inline]
    fn compressed(&self) -> bool {
        self.mime == MAGIC_COMPRESSED
    }
}

impl BankFsImpl {

    pub fn debinarize(
        reader: &mut impl SeekAndRead,
        options: BankReadOptions
    ) -> Result<Self, BankDebinarizationError> {
        let mut version_found: bool;
        let mut properties: HashMap<String, String> = HashMap::new();
        {
            let mut current_entry = EntryInfo::read_entry_meta(reader, options.max_entry_name_length)?;
            if options.require_version_entry && !current_entry.version() {
                return Err(BankDebinarizationError::FirstNotVersion)
            } else {
                version_found = true;
                read_properties(reader, &mut properties, options)
            }

        }
        todo!()
    }
}

pub enum DataLocationMethod {
    ///Respect offsets written inside of the entry meta 
    Deprecated,
    Calculate {
        ignore_impossible:     bool
    }

}

pub struct BankReadOptions {
    require_version_first:      bool,
    require_version_entry:      bool,
    respect_deprecated_offsets: DataLocationMethod,
    ignore_unused_properties:   bool,
    max_entry_count:            usize,
    max_entry_name_length:      usize,
    keep_empty_entries:         bool,
    allow_obfuscated:           bool,
    require_valid_checksum:     bool,
    max_property_length:        [usize; 2]
}

impl Default for DataLocationMethod {
    fn default() -> Self {
        Self::Calculate {
            ignore_impossible: true,
        }
    }
}

impl Default for BankReadOptions {
    fn default() -> Self {
        Self {
            require_version_first: true,
            require_version_entry: true,
            respect_deprecated_offsets: DataLocationMethod::default(),
            ignore_unused_properties: true,
            max_entry_count: usize::MAX,
            max_entry_name_length: 1024,
            keep_empty_entries: false,
            allow_obfuscated: false,
            require_valid_checksum: true,
            max_property_length: [1024, 1024],
        }
    }
}



fn read_properties(reader: &mut impl Read, properties: &mut HashMap<String, String>, options: BankReadOptions ) {
    loop {
        let name = read_utf8z(reader, options.max_property_length[0]);
        if name.is_empty() {
            return;
        }
        //TODO: Option: Ignore Unused Prefix
        let value = read_utf8z(reader, options.max_property_length[1]);
        properties.insert(name, value);
    }
}

fn read_utf8z(reader: &mut impl Read, cool_down: usize) -> String{
    let mut bytes = Vec::new();
    while bytes.len() < cool_down {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte).unwrap();
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes).unwrap()
}

#[derive(Debug)]
pub enum BankDebinarizationError {
    StringTooLong,
    FirstNotVersion,
    VersionNotFound,
    ImpossibleDataOffset,
    DecompressionError,
    DecryptionError,
    InvalidChecksum,
    Obfuscated,
    Other(String)
}
impl From<Box<dyn Error>> for BankDebinarizationError {
    fn from(err: Box<dyn Error>) -> BankDebinarizationError {
        BankDebinarizationError::Other(format!("An error occurred: {}", err))
    }
}

impl Display for BankDebinarizationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BankDebinarizationError::StringTooLong =>
            write!(f, "Bank Debinarization Error: A string has exceeded the maximum length defined in the debinarization options."),
            BankDebinarizationError::FirstNotVersion =>
                write!(f, "Bank Debinarization Error: The current options are configured to require a version entry to be the first in the bank. "),
            BankDebinarizationError::VersionNotFound =>
                write!(f, "Bank Debinarization Error: The current options are configured to require a version entry and none were found. "),
            BankDebinarizationError::ImpossibleDataOffset =>
                write!(f, "Bank Debinarization Error: The current options are configured to error out when offsets are invalidated. "),
            BankDebinarizationError::DecompressionError =>
                write!(f, "Bank Debinarization Error: There was an error while decompressing an entry. "),
            BankDebinarizationError::DecryptionError =>
                write!(f, "Bank Debinarization Error: There was an error while decrypting an entry. "),
            BankDebinarizationError::InvalidChecksum =>
                write!(f, "Bank Debinarization Error: The checksum does not match the one calculated. "),
            BankDebinarizationError::Obfuscated =>
                write!(f, "Bank Debinarization Error: The options are configured to prohibit obfuscated banks. "),
            BankDebinarizationError::Other(s) =>
                write!(f, "Bank Debinarization Error: {}", s),


        }
    }
}

impl Error for BankDebinarizationError {

}



//--------------------------------------------------------------------------------------------------
struct BankFsImpl {
    name:                 String,
    files:                HashMap<String, BankEntry>,
    properties:           HashMap<String, String>,
}

impl BankFsImpl {
    
    pub fn new(name: &str) -> Self {
        let mut files = HashMap::new();
        files.insert("".to_string(), BankEntry {
            file_type: VfsFileType::Directory,
            content: Arc::new(vec![])
        });

        let mut properties = HashMap::new();
        properties.insert(PROPERTY_PREFIX.to_string(), Self::normalize_path(name, true));


        Self { name: name.to_string(), files, properties, }
    }

    fn get_prefix(&self) -> String {
        self.properties.get(PROPERTY_PREFIX).unwrap_or(&self.name).clone()
    }

    fn get_prefix_mut(&mut self) -> &mut String {
        self.properties.get_mut(PROPERTY_PREFIX).unwrap_or(&mut self.name)
    }

    fn set_prefix(&mut self, new_prefix: String) -> Option<String> {
        self.properties.insert(PROPERTY_PREFIX.to_string(), Self::normalize_path(&*new_prefix, true))
    }

    fn relevize_and_normalize_path(&self, path: &str, directory: bool) -> String {
        self.relevize_path(&*Self::normalize_path(path, directory))
    }

    fn relevize_path(&self, path: &str) -> String {
        let prefix = format!("{}{}", self.get_prefix(), PATH_SEPARATOR);
        match path.starts_with(&prefix) {
            true => path[prefix.len()..].to_string(),
            false => path.to_string()
        }
    }

    fn normalize_path(path: &str, directory: bool) -> String {
        if path.is_empty() {
            return path.to_string();
        }

        let mut result = Vec::with_capacity(path.len());
        let mut last_was_separator = true;

        for c in path.chars() {
            match c {
                '/' | PATH_SEPARATOR => {
                    if last_was_separator { continue }

                    result.push(PATH_SEPARATOR);
                    last_was_separator = true;
                },
                _ => {
                    last_was_separator = false;
                    result.push(c.to_ascii_lowercase());
                }
            }
        }

        if !directory && !result.is_empty() && *result.last().unwrap() == PATH_SEPARATOR {
            result.pop();
        }

        result.iter().collect()
    }

}




//--------------------------------------------------------------------------------------------------
struct BankEntry {
    file_type:            VfsFileType,
    #[allow(clippy::rc_buffer)]
    content:              Arc<EntryContent>,
}




//--------------------------------------------------------------------------------------------------
struct WritableFile {
    content:              Cursor<EntryContent>,
    destination:          String,
    file_system:          BankFsHandle,
}

impl Write for WritableFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.content.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.content.flush()
    }
}

impl Drop for WritableFile {
    fn drop(&mut self) {
        let mut content = vec![];
        mem::swap(&mut content, self.content.get_mut());
        self.file_system.write().unwrap().files.insert(
            self.destination.clone(),
            BankEntry {
                file_type: VfsFileType::File,
                content: Arc::new(content),
            },
        );
    }
}




//--------------------------------------------------------------------------------------------------
struct ReadableFile  {
    #[allow(clippy::rc_buffer)]
    content:              Arc<EntryContent>,
    position:             u64
}

impl ReadableFile {
    fn remaining(&self) -> u64 {
        self.content.len() as u64 - self.position
    }
}

impl Read for ReadableFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = cmp::min(buf.len(), self.remaining() as usize);
        if amt == 1 {
            buf[0] = self.content[self.position as usize];
        } else {
            buf[..amt].copy_from_slice(
                &self.content.as_slice()[self.position as usize..self.position as usize + amt],
            );
        }
        self.position += amt as u64;
        Ok(amt)
    }
}

impl Seek for ReadableFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.position = offset,
            SeekFrom::Current(offset) => self.position = (self.position as i64 + offset) as u64,
            SeekFrom::End(offset) => self.position = (self.content.len() as i64 + offset) as u64,
        }
        Ok(self.position)
    }
}

fn ensure_file(file: &BankEntry) -> VfsResult<()> {
    if file.file_type != VfsFileType::File {
        return Err(VfsErrorKind::Other("Not a file".into()).into());
    }
    Ok(())
}

