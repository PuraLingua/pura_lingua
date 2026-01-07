use crate::item_token::TypeToken;
use global::StringName;
use global::attrs::TypeAttr;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::custom_attribute::CustomAttribute;
use crate::ty::GenericBounds;
use crate::ty::method::Method;

#[derive(ctor, Debug, Clone, Getters, CopyGetters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct InterfaceDef {
    pub(crate) type_vars: Vec<GenericBounds>,
    #[getset(skip)]
    #[get_copy = "pub"]
    pub(crate) attr: TypeAttr,
    pub(crate) custom_attributes: Vec<CustomAttribute>,
    pub(crate) name: StringName,
    pub(crate) satisfied_interfaces: Vec<TypeToken>,
    pub(crate) methods: Vec<Method>,
}
