use super::CORE_INSTANCE_SORT;
use crate::{encode_section, Encode, Export};

/// Represents an argument to a module instantiation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleArg {
    /// The argument is an instance.
    Instance(u32),
}

impl Encode for ModuleArg {
    fn encode(&self, sink: &mut Vec<u8>) {
        let (sort, idx) = match self {
            Self::Instance(idx) => (CORE_INSTANCE_SORT, *idx),
        };
        sink.push(sort);
        idx.encode(sink);
    }
}

/// An encoder for the core instance section of WebAssembly components.
///
/// # Example
///
/// ```rust
/// use wasm_encoder::{Component, InstanceSection, Export, ModuleArg};
///
/// let mut instances = InstanceSection::new();
/// instances.export_items([("foo", Export::Func(0))]);
/// instances.instantiate(1, [("foo", ModuleArg::Instance(0))]);
///
/// let mut component = Component::new();
/// component.section(&instances);
///
/// let bytes = component.finish();
/// ```
#[derive(Clone, Debug, Default)]
pub struct InstanceSection {
    bytes: Vec<u8>,
    num_added: u32,
}

impl InstanceSection {
    /// Create a new core instance section encoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of instances in the section.
    pub fn len(&self) -> u32 {
        self.num_added
    }

    /// Determines if the section is empty.
    pub fn is_empty(&self) -> bool {
        self.num_added == 0
    }

    /// Define an instance by instantiating a core module.
    pub fn instantiate<'a, A>(&mut self, module_index: u32, args: A) -> &mut Self
    where
        A: IntoIterator<Item = (&'a str, ModuleArg)>,
        A::IntoIter: ExactSizeIterator,
    {
        let args = args.into_iter();
        self.bytes.push(0x00);
        module_index.encode(&mut self.bytes);
        args.len().encode(&mut self.bytes);
        for (name, arg) in args {
            name.encode(&mut self.bytes);
            arg.encode(&mut self.bytes);
        }
        self.num_added += 1;
        self
    }

    /// Define an instance by exporting core WebAssembly items.
    pub fn export_items<'a, E>(&mut self, exports: E) -> &mut Self
    where
        E: IntoIterator<Item = (&'a str, Export)>,
        E::IntoIter: ExactSizeIterator,
    {
        let exports = exports.into_iter();
        self.bytes.push(0x01);
        exports.len().encode(&mut self.bytes);
        for (name, export) in exports {
            name.encode(&mut self.bytes);
            export.encode(&mut self.bytes);
        }
        self.num_added += 1;
        self
    }
}

impl Encode for InstanceSection {
    fn encode(&self, sink: &mut Vec<u8>) {
        encode_section(sink, self.num_added, &self.bytes);
    }
}
