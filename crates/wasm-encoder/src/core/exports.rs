use super::{
    CORE_FUNCTION_SORT, CORE_GLOBAL_SORT, CORE_MEMORY_SORT, CORE_TABLE_SORT, CORE_TAG_SORT,
};
use crate::{encode_section, Encode, Section, SectionId};

/// Represents the kind of an export from a WebAssembly module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportKind {
    /// The export is a function.
    Func,
    /// The export is a table.
    Table,
    /// The export is a memory.
    Memory,
    /// The export is a global.
    Global,
    /// The export is a tag.
    Tag,
}

impl Encode for ExportKind {
    fn encode(&self, sink: &mut Vec<u8>) {
        sink.push(match self {
            Self::Func => CORE_FUNCTION_SORT,
            Self::Table => CORE_TABLE_SORT,
            Self::Memory => CORE_MEMORY_SORT,
            Self::Global => CORE_GLOBAL_SORT,
            Self::Tag => CORE_TAG_SORT,
        });
    }
}

/// Represents an export from a WebAssembly module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Export {
    /// The export is a function.
    Func(u32),
    /// The export is a table.
    Table(u32),
    /// The export is a memory.
    Memory(u32),
    /// The export is a global.
    Global(u32),
    /// The export is a tag.
    ///
    /// This variant is used with the exception handling proposal.
    Tag(u32),
}

impl Export {
    /// Gets the kind of the export.
    pub fn kind(&self) -> ExportKind {
        match self {
            Self::Func(_) => ExportKind::Func,
            Self::Table(_) => ExportKind::Table,
            Self::Memory(_) => ExportKind::Memory,
            Self::Global(_) => ExportKind::Global,
            Self::Tag(_) => ExportKind::Tag,
        }
    }

    /// Gets the index of the export.
    fn index(&self) -> u32 {
        match self {
            Self::Func(i) | Self::Table(i) | Self::Memory(i) | Self::Global(i) | Self::Tag(i) => *i,
        }
    }
}

impl Encode for Export {
    fn encode(&self, sink: &mut Vec<u8>) {
        self.kind().encode(sink);
        self.index().encode(sink);
    }
}

/// An encoder for the export section of WebAssembly module.
///
/// # Example
///
/// ```rust
/// use wasm_encoder::{Module, ExportSection, Export};
///
/// let mut exports = ExportSection::new();
/// exports.export("foo", Export::Func(0));
///
/// let mut module = Module::new();
/// module.section(&exports);
///
/// let bytes = module.finish();
/// ```
#[derive(Clone, Debug, Default)]
pub struct ExportSection {
    bytes: Vec<u8>,
    num_added: u32,
}

impl ExportSection {
    /// Create a new export section encoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of exports in the section.
    pub fn len(&self) -> u32 {
        self.num_added
    }

    /// Determines if the section is empty.
    pub fn is_empty(&self) -> bool {
        self.num_added == 0
    }

    /// Define an export in the export section.
    pub fn export(&mut self, name: &str, export: Export) -> &mut Self {
        name.encode(&mut self.bytes);
        export.encode(&mut self.bytes);
        self.num_added += 1;
        self
    }
}

impl Encode for ExportSection {
    fn encode(&self, sink: &mut Vec<u8>) {
        encode_section(sink, self.num_added, &self.bytes);
    }
}

impl Section for ExportSection {
    fn id(&self) -> u8 {
        SectionId::Export.into()
    }
}
