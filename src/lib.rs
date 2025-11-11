// Library interface for localdoc CLI
// This exposes modules for testing purposes

pub mod docpack;
pub mod godot_parser;
pub mod lister;
pub mod packer;
pub mod query;

// Re-export commonly used items for convenience
pub use docpack::{DocEntry, EntryType, Manifest};
pub use godot_parser::parse_godot_xml;
pub use packer::pack_godot_docs;
pub use query::{query_docpacks, QueryOptions, SearchResult};
