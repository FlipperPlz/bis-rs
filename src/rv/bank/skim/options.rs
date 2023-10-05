use crate::DebinarizationOptions;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum OffsetLocationStrategy {
    Deprecated,
    Calculate
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BankSkimOptions {
    pub(crate) offset_location_strategy:   OffsetLocationStrategy,
    pub(crate) allow_offsets_to_header:    bool,
    pub(crate) remove_impossible_offsets:  bool,
    pub(crate) require_version_first:      bool,
    pub(crate) require_version_entry:      bool,
    pub(crate) ignore_unused_properties:   bool,
    pub(crate) max_entry_count:            usize,
    pub(crate) remove_empty_entries:       bool,
    pub(crate) allow_obfuscated:           bool,
    pub(crate) require_valid_checksum:     bool,
}

impl Default for BankSkimOptions {
    fn default() -> Self {
        Self {
            offset_location_strategy: OffsetLocationStrategy::Deprecated,
            allow_offsets_to_header: false,
            remove_impossible_offsets: false,
            require_version_first: false,
            require_version_entry: false,
            ignore_unused_properties: false,
            max_entry_count: usize::MAX,
            remove_empty_entries: false,
            allow_obfuscated: false,
            require_valid_checksum: false,
        }
    }
}

impl DebinarizationOptions for BankSkimOptions {

}

