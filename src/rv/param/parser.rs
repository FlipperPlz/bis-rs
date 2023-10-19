use std::fs::File;
use std::path::Path;
use std::io::Read;

enum Token {
    ClassKeyword,
    EnumKeyword,

}

pub fn parse_virtual_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    todo!("rv::vfs")
}

pub async fn parse_system_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    parse(&buffer);
    Ok(())
}

pub fn parse(content: &[u8]) {

}