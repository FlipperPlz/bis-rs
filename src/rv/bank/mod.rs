use std::borrow::ToOwned;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

const MAGIC_DECOMPRESSED: i32    = 0x00000000;
const MAGIC_COMPRESSED:   i32    = 0x43707273;
const MAGIC_ENCRYPTED:    i32    = 0x456e6372;
const MAGIC_VERSION:      i32    = 0x56657273;
const PROPERTY_PREFIX:    String = String::from("prefix");
pub type NodeID = usize;

pub struct Archive {
    name:         String,
    nodes:        HashMap<NodeID, Node>,
    properties:   HashMap<String, String>,
    root:         NodeID
}

pub struct FolderNode {
    name:         String,
    archive:      Arc<Archive>,
    parent:       NodeID,
    children:     Vec<NodeID>
}

pub struct FileNode {
    name:         String,
    archive:      Arc<Archive>,
    parent:       NodeID,
    format:       DataFormat,
    file_offset:  usize,
    written_size: usize,
    size:         i32,
}

pub struct NodeIterator<'a> {
    archive:      &'a Archive,
    stack:        Vec<NodeID>
}

pub enum Node {
    File(FileNode),
    Folder(FolderNode),
}

pub enum DataFormat {
    Decompressed,
    Compressed,
    Encrypted
}

#[derive(Debug)]
pub enum EntryError {
    NodeNotFound,
}

impl DataFormat {
    pub const fn magic(&self) -> i32 {
        match self {
            DataFormat::Decompressed => MAGIC_DECOMPRESSED,
            DataFormat::Compressed => MAGIC_COMPRESSED,
            DataFormat::Encrypted => MAGIC_ENCRYPTED,
        }
    }
}

impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntryError::NodeNotFound => write!(f, "Entry not found in the current context."),
        }
    }
}

impl Error for EntryError {}

impl Archive {
    pub fn get_prefix(&self) -> &String {
        match self.properties.get(&*PROPERTY_PREFIX) {
            None => &self.name,
            Some(it) => it
        }
    }

    pub fn set_prefix(&mut self, prefix: &str) -> Option<String> {
        self.properties.insert(PROPERTY_PREFIX, prefix.to_owned())
    }

    pub fn get_node(&self, id: NodeID) -> Option<&Node> {
        match self.nodes.get(&id) {
            None => None,
            Some(it) => it
        }
    }

    pub fn get_directory_node(&self, id: NodeID) -> Option<&FolderNode> {
        self.get_node(id).and_then(|node| {
            match node {
                Node::File(_) => None,
                Node::Folder(folder) => Some(folder),
            }
        })
    }

    pub fn get_file_node(&self, id: NodeID) -> Option<&FileNode> {
        self.get_node(id).and_then(|node| {
            match node {
                Node::File(file) => Some(file),
                Node::Folder(_) => None,
            }
        })
    }
}

impl Archive {
    pub fn get_root(&self) -> &FolderNode {
        self.get_directory_node(self.root)?
    }
}

impl Node {
    pub fn get_parent(&self) -> &FolderNode {
        let (archive, parent) = match self {
            Node::File(file) => (&file.archive, file.parent),
            Node::Folder(folder) => (&folder.archive, folder.parent)
        };

        archive.get_directory_node(parent)?
    }

    pub fn get_name(&self) -> &String {
        match self {
            Node::File(file) => &file.name,
            Node::Folder(folder) => &folder.name
        }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.archive.get_node(self.stack.pop()?)
    }
}

impl<'a> IntoIterator for &'a FolderNode {
    type Item = &'a Node;
    type IntoIter = NodeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iterator = NodeIterator {
            archive: &Arc::as_ref(&self.archive),
            stack: vec![]
        };
        iterator.stack.extend(self.children.iter());

        iterator
    }
}

impl<'a> IntoIterator for &'a Archive {
    type Item = &'a Node;
    type IntoIter = NodeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.get_root() {
            Some(Node::Folder(folder)) => NodeIterator {
                archive: self,
                stack: folder.children.iter().collect(),
            },
            _ => NodeIterator {
                archive: self,
                stack: vec![],
            }
        }
    }
}



