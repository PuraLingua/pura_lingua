use binary_core::traits::StringRef;
use global::attrs::TypeAttr;
use proc_macros::{ReadFromSection, WriteToSection};

use super::{Field, GenericBounds, Method};

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct StructDef {
    pub name: StringRef,
    pub attr: TypeAttr,

    // Note that Struct does not have parents
    pub method_table: Vec<Method>,
    pub fields: Vec<Field>,
    pub sctor: Option<u32>,

    pub generic_bounds: Option<Vec<GenericBounds>>,
}
