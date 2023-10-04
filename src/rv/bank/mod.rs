pub mod io;

use std::{cmp, fmt, mem};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, RwLock};
use vfs::{FileSystem, SeekAndRead, VfsFileType, VfsMetadata, VfsResult};
use vfs::error::VfsErrorKind;

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
struct BankFsImpl {
    name:             String,
    properties:       HashMap<String, String>,
    files:                HashMap<String, BankEntry>,
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

//--------------------------------------------------------------------------------------------------
