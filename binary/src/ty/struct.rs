use crate::item_token::TypeToken;
use global::attrs::TypeAttr;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::{IndexMap, StringName};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::custom_attribute::CustomAttribute;
use crate::field::Field;
use crate::ty::GenericBounds;
use crate::ty::method::Method;

#[derive(ctor, Debug, Clone, Getters, CopyGetters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct StructDef {
    pub(crate) parent: Option<TypeToken>,
    pub(crate) type_vars: IndexMap<StringName, GenericBounds>,
    #[getset(skip)]
    #[get_copy = "pub"]
    pub(crate) attr: TypeAttr,
    pub(crate) custom_attributes: Vec<CustomAttribute>,
    pub(crate) name: StringName,
    pub(crate) methods: Vec<Method>,
    pub(crate) fields: IndexMap<StringName, Field>,
}
