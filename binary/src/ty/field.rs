use crate::item_token::TypeToken;
use global::StringName;
use global::attrs::FieldAttr;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::custom_attribute::CustomAttribute;

#[derive(Clone, Debug, Getters, CopyGetters, ReadFromFile, WriteToFile, ctor)]
#[getset(get = "pub")]
pub struct Field {
    pub(crate) name: StringName,
    #[getset(skip)]
    #[get_copy = "pub"]
    pub(crate) attr: FieldAttr,
    pub(crate) custom_attributes: Vec<CustomAttribute>,
    pub(crate) ty: TypeToken,
}
