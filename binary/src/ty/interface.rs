use binary_core::traits::StringRef;
use global::attrs::TypeAttr;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::{item_token::TypeToken, ty::GenericCountRequirement};

use super::{GenericBounds, Method};

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct InterfaceDef {
    pub name: StringRef,
    pub attr: TypeAttr,

    pub generic_count_requirement: GenericCountRequirement,

    pub required_interfaces: Vec<TypeToken>,

    pub method_table: Vec<Method>,

    pub generic_bounds: Option<Vec<GenericBounds>>,
}

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct InterfaceImplementation {
    pub target: TypeToken,

    pub map: Vec<u32>,
}
