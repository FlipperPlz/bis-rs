
pub enum OffsetLocationStrategy {
    Deprecated,
    Calculate
}


pub enum MultipleVersionStrategy {
    Forbid,
    Allow {
        should_read_props:       bool
    }
}

pub struct BankSkimOptions {
    pub(crate) multiple_version_strategy:  MultipleVersionStrategy,
    pub(crate) offset_location_strategy:   OffsetLocationStrategy,
    pub(crate) allow_offsets_to_header:    bool,
    pub(crate) remove_impossible_offsets:  bool,
    pub(crate) require_version_first:      bool,
    pub(crate) require_version_entry:      bool,
    pub(crate) require_version_blanked:    bool,
    pub(crate) ignore_unused_properties:   bool,
    pub(crate) max_entry_count:            usize,
    pub(crate) max_entry_name_length:      usize,
    pub(crate) remove_empty_entries:       bool,
    pub(crate) allow_obfuscated:           bool,
    pub(crate) require_valid_checksum:     bool,
    pub(crate) max_property_length:        [usize; 2]
}

