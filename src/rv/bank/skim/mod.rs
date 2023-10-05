pub mod options; pub use options::*;
pub mod entry; pub use entry::*;

use std::collections::HashMap;
pub struct BankFile {
    pub(crate) header_start:  u64,
    pub(crate) buffer_start:  u64,
    pub(crate) entries:    Vec<BankEntry>,
    pub(crate) properties: HashMap<String, String>
}