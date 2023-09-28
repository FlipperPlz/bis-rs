use std::borrow::ToOwned;
use std::collections::HashMap;
use std::error::Error;

const MAGIC_DECOMPRESSED: i32    = 0x00000000;
const MAGIC_COMPRESSED:   i32    = 0x43707273;
const MAGIC_ENCRYPTED:    i32    = 0x456e6372;
const MAGIC_VERSION:      i32    = 0x56657273;
const PROPERTY_PREFIX:    &str   = "prefix";
pub type NodeID =         usize;

pub struct Archive {
    name:         String,
    nodes:        HashMap<NodeID, Node>,
    properties:   HashMap<String, String>,
    root:         NodeID
}

pub struct FolderNode {
    name:         String,
    parent:       NodeID,
    children:     Vec<NodeID>
}

pub struct FileNode {
    name:         String,
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
    pub fn iter(&self) -> NodeIterator {
        self.into_iter()
    }

    pub fn iter_folder(&self, folder: &FolderNode) -> NodeIterator {
        if let Some(first_child) = folder.children.first() {
            NodeIterator::new(self, *first_child)
        } else {
            NodeIterator::new(self, self.root)
        }
    }

    pub fn get_prefix(&self) -> &String {
        match self.properties.get(&*PROPERTY_PREFIX) {
            None => &self.name,
            Some(it) => it
        }
    }

    pub fn set_prefix(&mut self, prefix: &str) -> Option<String> {
        self.properties.insert(PROPERTY_PREFIX.to_string(), prefix.to_owned())
    }

    pub fn get_node(&self, id: NodeID) -> Option<&Node> {
        self.nodes.get(&id)
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

    pub fn get_root_node(&self) -> &FolderNode {
        self.get_directory_node(self.root).unwrap()
    }
}

impl Node {
    pub fn get_parent_node_id(&self) -> NodeID {
        match self {
            Node::File(file) => file.parent,
            Node::Folder(folder) => folder.parent
        }
    }
    pub fn get_parent_node<'a>(&self, archive: &'a Archive) -> Result<&'a FolderNode, EntryError> {
        match archive.get_directory_node(self.get_parent_node_id()) {
            Some(folder) => Ok(folder),
            None => Err(EntryError::NodeNotFound),
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Node::File(file) => &file.name,
            Node::Folder(folder) => &folder.name
        }
    }
}

impl<'a> NodeIterator<'a> {

    pub fn new(archive: &'a Archive, start: NodeID) -> NodeIterator<'a> {
        NodeIterator {
            archive,
            stack: vec![start],
        }
    }

    fn push_children(&mut self, node_id: NodeID) {
        if let Some(node) = self.archive.get_node(node_id) {
            if let Node::Folder(folder) = node {
                for child_id in &folder.children {
                    self.stack.push(*child_id);
                }
            }
        }
    }
}

impl<'a> IntoIterator for &'a Archive {
    type Item = &'a Node;
    type IntoIter = NodeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        NodeIterator::new(self, self.root)
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let next_node_id = self.stack.pop()?;
        self.push_children(next_node_id);
        self.archive.get_node(next_node_id)
    }
}