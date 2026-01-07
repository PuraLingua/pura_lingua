use crate::item_token::{MethodToken, TypeToken};
use global::StringName;
use global::attrs::{CallConvention, MethodAttr, ParameterAttr};
use global::derive_ctor::ctor;
use global::getset::Getters;
use global::instruction::Instruction;
use proc_macros::{ReadFromFile, WriteToFile};

use crate::custom_attribute::CustomAttribute;
use crate::ty::GenericBounds;

#[derive(Clone, Debug, Getters, ctor, ReadFromFile, WriteToFile)]
#[allow(unused)]
#[getset(get = "pub")]
#[ctor(pub new)]
pub struct MethodSign {
    convention: CallConvention,
    args: Vec<(TypeToken, ParameterAttr)>,
    ret_type: TypeToken,
}

#[derive(Clone, Debug, Getters, ctor, ReadFromFile, WriteToFile)]
#[allow(unused)]
#[getset(get = "pub")]
#[ctor(pub new)]
pub struct Method {
    attr: MethodAttr<TypeToken>,
    sign: MethodSign,
    name: StringName,

    custom_attributes: Vec<CustomAttribute>,
    instructions: Vec<Instruction<TypeToken, MethodToken, u32>>,
    type_vars: Vec<GenericBounds>,
}

#[derive(Clone, Debug, Getters, ReadFromFile, WriteToFile, PartialEq)]
#[getset(get = "pub")]
pub struct MethodSpec {
    m: u32,
    generics: Vec<TypeToken>,
}
