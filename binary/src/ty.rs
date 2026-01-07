use crate::item_token::TypeToken;
use global::attrs::TypeAttr;
use global::getset::Getters;
use global::{StringName, WithType};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::interface::InterfaceDef;
use crate::ty::class::ClassDef;
use crate::ty::r#struct::StructDef;

pub mod class;
pub mod field;
pub mod interface;
pub mod method;
pub mod r#struct;

#[derive(Debug, Clone, WithType, ReadFromFile, WriteToFile)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromFile, WriteToFile))]
#[allow(clippy::large_enum_variant)]
pub enum TypeDef {
    Class(ClassDef),
    Struct(StructDef),
    Interface(InterfaceDef),
}

impl TypeDef {
    pub fn name(&self) -> &StringName {
        match self {
            TypeDef::Class(class_def) => class_def.name(),
            TypeDef::Struct(struct_def) => struct_def.name(),
            TypeDef::Interface(interface_def) => interface_def.name(),
        }
    }
    pub fn attr(&self) -> TypeAttr {
        match self {
            TypeDef::Class(class_def) => class_def.attr,
            TypeDef::Struct(struct_def) => struct_def.attr,
            TypeDef::Interface(interface_def) => interface_def.attr,
        }
    }
}

#[derive(Clone, Debug, Getters, ReadFromFile, WriteToFile, PartialEq)]
#[getset(get = "pub")]
pub struct TypeRef {
    assembly: StringName,
    index: u32,
}

#[derive(Clone, Debug, Getters, ReadFromFile, WriteToFile, PartialEq)]
#[getset(get = "pub")]
pub struct TypeSpec {
    ty: TypeToken,
    generics: Vec<TypeToken>,
}

#[derive(Clone, Debug, Getters, ReadFromFile, WriteToFile, PartialEq)]
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

#[repr(u8)]
pub enum ElementType {
    Void,
    Boolean,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Usize,
    Isize,
    String,
    Object,
    Pointer(Box<ElementType>),
    ByRef(Box<ElementType>),
    ValueType(TypeToken),
    Class(TypeToken),
    TypeVar(u32),
    Array(TypeToken),
    GenericInst(TypeToken, Vec<TypeToken>),
}
