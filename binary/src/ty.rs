use binary_core::traits::StringRef;
use global::{UnwrapEnum, WithType, attrs::TypeAttr, getset::Getters};
use proc_macros::{ReadFromSection, WriteToSection};

use crate::item_token::TypeToken;

mod class;
mod field;
mod method;
mod r#struct;

pub use class::ClassDef;
pub use field::Field;
pub use method::{BinaryInstruction, Method, MethodSpec, Parameter};
pub use r#struct::StructDef;

#[derive(Debug, Clone, WithType, UnwrapEnum, ReadFromSection, WriteToSection)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromSection, WriteToSection))]
#[allow(clippy::large_enum_variant)]
#[unwrap_enum(ref, ref_mut, owned)]
pub enum TypeDef {
    Class(ClassDef),
    Struct(StructDef),
    // Interface(InterfaceDef),
}

impl TypeDef {
    pub fn name(&self) -> &StringRef {
        match self {
            TypeDef::Class(class_def) => &class_def.name,
            TypeDef::Struct(struct_def) => &struct_def.name,
            // TypeDef::Interface(interface_def) => interface_def.name(),
        }
    }
    pub fn attr(&self) -> TypeAttr {
        match self {
            TypeDef::Class(class_def) => class_def.attr,
            TypeDef::Struct(struct_def) => struct_def.attr,
            // TypeDef::Interface(interface_def) => interface_def.attr,
        }
    }
}

#[derive(Clone, Debug, Getters, ReadFromSection, WriteToSection, PartialEq)]
#[getset(get = "pub")]
pub struct TypeRef {
    pub assembly: StringRef,
    pub index: u32,
}

#[derive(Clone, Debug, Getters, ReadFromSection, WriteToSection, PartialEq)]
#[getset(get = "pub")]
pub struct TypeSpec {
    pub ty: TypeToken,
    pub generics: Vec<TypeToken>,
}

#[derive(Clone, Debug, Getters, ReadFromSection, WriteToSection, PartialEq)]
#[getset(get = "pub")]
pub struct GenericBounds {
    pub implemented_interfaces: Vec<TypeToken>,
    pub parent: Option<TypeToken>,
}

impl GenericBounds {
    pub fn new(implemented_interfaces: Vec<TypeToken>, parent: Option<TypeToken>) -> Self {
        Self {
            implemented_interfaces,
            parent,
        }
    }
}
