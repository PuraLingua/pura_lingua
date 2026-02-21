use binary_core::traits::StringRef;
use global::attrs::FieldAttr;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::item_token::TypeToken;

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Field {
    pub name: StringRef,
    pub attr: FieldAttr,
    pub ty: TypeToken,
}
