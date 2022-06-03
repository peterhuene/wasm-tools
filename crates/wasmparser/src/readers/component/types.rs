use crate::{
    BinaryReader, ComponentAlias, ComponentImport, ComponentTypeRef, Result,
    SectionIteratorLimited, SectionReader, SectionWithLimitedItems, Type,
};
use std::ops::Range;

/// Represents a value type in a WebAssembly component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentValType {
    /// The value type is a primitive type.
    Primitive(PrimitiveValType),
    /// The value type is a reference to a defined type.
    Type(u32),
}

/// Represents a primitive value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveValType {
    /// The type is the unit type.
    Unit,
    /// The type is a boolean.
    Bool,
    /// The type is a signed 8-bit integer.
    S8,
    /// The type is an unsigned 8-bit integer.
    U8,
    /// The type is a signed 16-bit integer.
    S16,
    /// The type is an unsigned 16-bit integer.
    U16,
    /// The type is a signed 32-bit integer.
    S32,
    /// The type is an unsigned 32-bit integer.
    U32,
    /// The type is a signed 64-bit integer.
    S64,
    /// The type is an unsigned 64-bit integer.
    U64,
    /// The type is a 32-bit floating point number.
    Float32,
    /// The type is a 64-bit floating point number.
    Float64,
    /// The type is a Unicode character.
    Char,
    /// The type is a string.
    String,
}

impl PrimitiveValType {
    pub(crate) fn requires_realloc(&self) -> bool {
        matches!(self, Self::String)
    }

    pub(crate) fn is_subtype_of(&self, other: &Self) -> bool {
        // Subtyping rules according to
        // https://github.com/WebAssembly/component-model/blob/17f94ed1270a98218e0e796ca1dad1feb7e5c507/design/mvp/Subtyping.md
        self == other
            || matches!(
                (self, other),
                (_, Self::Unit)
                    | (Self::S8, Self::S16)
                    | (Self::S8, Self::S32)
                    | (Self::S8, Self::S64)
                    | (Self::U8, Self::U16)
                    | (Self::U8, Self::U32)
                    | (Self::U8, Self::U64)
                    | (Self::U8, Self::S16)
                    | (Self::U8, Self::S32)
                    | (Self::U8, Self::S64)
                    | (Self::S16, Self::S32)
                    | (Self::S16, Self::S64)
                    | (Self::U16, Self::U32)
                    | (Self::U16, Self::U64)
                    | (Self::U16, Self::S32)
                    | (Self::U16, Self::S64)
                    | (Self::S32, Self::S64)
                    | (Self::U32, Self::U64)
                    | (Self::U32, Self::S64)
                    | (Self::Float32, Self::Float64)
            )
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Unit => 0,
            _ => 1,
        }
    }
}

/// Represents a type in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum ComponentType<'a> {
    /// The type is a component defined type.
    Defined(ComponentDefinedType<'a>),
    /// The type is a function type.
    Func(ComponentFuncType<'a>),
    /// The type is a component type.
    Component(Box<[ComponentTypeDeclaration<'a>]>),
    /// The type is an instance type.
    Instance(Box<[InstanceTypeDeclaration<'a>]>),
}

/// Represents part of a component type declaration in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum ComponentTypeDeclaration<'a> {
    /// The component type declaration is for a core type.
    CoreType(Type<'a>),
    /// The component type declaration is for a type.
    Type(ComponentType<'a>),
    /// The component type declaration is for an alias.
    Alias(ComponentAlias<'a>),
    /// The component type declaration is for an export.
    Export {
        /// The name of the export.
        name: &'a str,
        /// The type reference for the export.
        ty: ComponentTypeRef,
    },
    /// The component type declaration is for an import.
    Import(ComponentImport<'a>),
}

/// Represents an instance type declaration in a WebAssembly component.
#[derive(Debug, Clone)]
pub enum InstanceTypeDeclaration<'a> {
    /// The component type declaration is for a core type.
    CoreType(Type<'a>),
    /// The instance type declaration is for a type.
    Type(ComponentType<'a>),
    /// The instance type declaration is for an alias.
    Alias(ComponentAlias<'a>),
    /// The instance type declaration is for an export.
    Export {
        /// The name of the export.
        name: &'a str,
        /// The type reference for the export.
        ty: ComponentTypeRef,
    },
}

/// Represents a type of a function in a WebAssembly component.
#[derive(Debug, Clone)]
pub struct ComponentFuncType<'a> {
    /// The function parameter types.
    pub params: Box<[(Option<&'a str>, ComponentValType)]>,
    /// The function result type.
    pub result: ComponentValType,
}

/// Represents a case in a variant type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantCase<'a> {
    /// The name of the variant case.
    pub name: &'a str,
    /// The value type of the variant case.
    pub ty: ComponentValType,
    /// The default-to case index to use when this case is not present.
    pub default_to: Option<u32>,
}

/// Represents a defined type in a WebAssembly component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentDefinedType<'a> {
    /// The type is one of the primitive value types.
    Primitive(PrimitiveValType),
    /// The type is a record with the given fields.
    Record(Box<[(&'a str, ComponentValType)]>),
    /// The type is a variant with the given cases.
    Variant(Box<[VariantCase<'a>]>),
    /// The type is a list of the given value type.
    List(ComponentValType),
    /// The type is a tuple of the given value types.
    Tuple(Box<[ComponentValType]>),
    /// The type is flags with the given names.
    Flags(Box<[&'a str]>),
    /// The type is an enum with the given tags.
    Enum(Box<[&'a str]>),
    /// The type is a union of the given value types.
    Union(Box<[ComponentValType]>),
    /// The type is an option of the given value type.
    Option(ComponentValType),
    /// The type is an expected type.
    Expected {
        /// The type returned for success.
        ok: ComponentValType,
        /// The type returned for failure.
        error: ComponentValType,
    },
}

/// A reader for the type section of a WebAssembly component.
#[derive(Clone)]
pub struct ComponentTypeSectionReader<'a> {
    reader: BinaryReader<'a>,
    count: u32,
}

impl<'a> ComponentTypeSectionReader<'a> {
    /// Constructs a new `ComponentTypeSectionReader` for the given data and offset.
    pub fn new(data: &'a [u8], offset: usize) -> Result<Self> {
        let mut reader = BinaryReader::new_with_offset(data, offset);
        let count = reader.read_var_u32()?;
        Ok(Self { reader, count })
    }

    /// Gets the original position of the reader.
    pub fn original_position(&self) -> usize {
        self.reader.original_position()
    }

    /// Gets a count of items in the section.
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// Reads content of the type section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentTypeSectionReader;
    /// let data: &[u8] = &[0x01, 0x40, 0x01, 0x01, 0x03, b'f', b'o', b'o', 0x72, 0x72];
    /// let mut reader = ComponentTypeSectionReader::new(data, 0).unwrap();
    /// for _ in 0..reader.get_count() {
    ///     let ty = reader.read().expect("type");
    ///     println!("Type {:?}", ty);
    /// }
    /// ```
    pub fn read(&mut self) -> Result<ComponentType<'a>> {
        self.reader.read_component_type()
    }
}

impl<'a> SectionReader for ComponentTypeSectionReader<'a> {
    type Item = ComponentType<'a>;

    fn read(&mut self) -> Result<Self::Item> {
        Self::read(self)
    }

    fn eof(&self) -> bool {
        self.reader.eof()
    }

    fn original_position(&self) -> usize {
        Self::original_position(self)
    }

    fn range(&self) -> Range<usize> {
        self.reader.range()
    }
}

impl<'a> SectionWithLimitedItems for ComponentTypeSectionReader<'a> {
    fn get_count(&self) -> u32 {
        Self::get_count(self)
    }
}

impl<'a> IntoIterator for ComponentTypeSectionReader<'a> {
    type Item = Result<ComponentType<'a>>;
    type IntoIter = SectionIteratorLimited<Self>;

    /// Implements iterator over the type section.
    ///
    /// # Examples
    /// ```
    /// use wasmparser::ComponentTypeSectionReader;
    /// let data: &[u8] = &[0x01, 0x40, 0x01, 0x01, 0x03, b'f', b'o', b'o', 0x72, 0x72];
    /// let mut reader = ComponentTypeSectionReader::new(data, 0).unwrap();
    /// for ty in reader {
    ///     println!("Type {:?}", ty.expect("type"));
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        SectionIteratorLimited::new(self)
    }
}
