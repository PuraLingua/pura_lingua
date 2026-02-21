use binary_core::traits::StringRef;
use global::attrs::TypeAttr;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::item_token::TypeToken;

use super::{Field, GenericBounds, Method};

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct ClassDef {
    pub main: Option<u32>,

    pub name: StringRef,
    pub attr: TypeAttr,

    pub parent: Option<TypeToken>,

    pub method_table: Vec<Method>,
    pub fields: Vec<Field>,
    pub sctor: Option<u32>,

    pub generic_bounds: Option<Vec<GenericBounds>>,
}
