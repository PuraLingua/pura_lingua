use binary_proc_macros::{ReadFromSection, WriteToSection};
use derive_ctor::ctor;
use enumflags2::{BitFlags, bitflags};
use getset::{CopyGetters, MutGetters, Setters};
use global_proc_macros::{UnwrapEnum, WithType};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::Visibility;

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    UnwrapEnum,
    WithType,
    Eq,
    PartialEq,
    ReadFromSection,
    WriteToSection,
    serde::Serialize,
    serde::Deserialize,
)]
#[with_type(derive = (
    TryFromPrimitive,
    IntoPrimitive,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ReadFromSection,
    WriteToSection,
    // std::marker::ConstParamTy,
    serde::Serialize,
    serde::Deserialize,
))]
#[unwrap_enum(ref, ref_mut, owned)]
pub enum TypeSpecificAttr {
    Class(BitFlags<ClassImplementationFlags>),
    Struct(BitFlags<StructImplementationFlags>),
    Interface(BitFlags<InterfaceImplementationFlags>),
}

impl TypeSpecificAttr {
    pub fn is_partial(&self) -> bool {
        match self {
            TypeSpecificAttr::Class(flags) => flags.contains(ClassImplementationFlags::Partial),
            TypeSpecificAttr::Struct(flags) => flags.contains(StructImplementationFlags::Partial),
            TypeSpecificAttr::Interface(_flags) => false,
        }
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StructImplementationFlags {
    Ref,
    Partial,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassImplementationFlags {
    Static,
    Partial,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterfaceImplementationFlags {
    __,
}

#[repr(C)]
#[derive(
    Clone,
    Copy,
    Debug,
    ctor,
    CopyGetters,
    Setters,
    MutGetters,
    Eq,
    PartialEq,
    ReadFromSection,
    WriteToSection,
    serde::Serialize,
    serde::Deserialize,
)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
pub struct TypeAttr {
    vis: Visibility,
    specific: TypeSpecificAttr,
}
