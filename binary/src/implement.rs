use crate::method::Method;

use crate::item_token::TypeToken;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::{IndexMap, StringName};
use proc_macros::{ReadFromFile, WriteToFile};

#[derive(ctor, Debug, Clone, Getters, CopyGetters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct Implementation {
    ty: TypeToken,
    interface: Option<TypeToken>,
    methods: IndexMap<StringName, Method>,
}
