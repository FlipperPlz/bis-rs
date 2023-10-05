
pub enum OffsetLocationStrategy {
    Deprecated,
    Calculate
}

#[derive(PartialEq)]
pub enum ErrorStrategy {
    Allow,
    Deny,
    Ignore
}

pub enum MultipleVersionStrategy {
    Forbid,
    Allow {
        should_read_props:       bool
    }
}

pub struct BankSkimOptions {
    multiple_version_strategy:  MultipleVersionStrategy,
    offset_location_strategy:   OffsetLocationStrategy,
    allow_offsets_to_header:    ErrorStrategy,
    require_version_first:      bool,
    require_version_entry:      bool,
    require_version_blanked:    bool,
    ignore_unused_properties:   bool,
    max_entry_count:            usize,
    max_entry_name_length:      usize,
    remove_empty_entries:       bool,
    allow_obfuscated:           bool,
    require_valid_checksum:     bool,
    max_property_length:        [usize; 2]
}

