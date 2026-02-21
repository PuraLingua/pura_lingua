use binary_core::traits::StringRef;
use global::{
    attrs::{CallConvention, MethodAttr, ParameterAttr},
    instruction::Instruction,
};
use proc_macros::{ReadFromSection, WriteToSection};

use crate::item_token::{MethodToken, TypeToken};

use super::GenericBounds;

pub type BinaryInstruction = Instruction<StringRef, TypeToken, MethodToken, u32>;

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Method {
    pub name: StringRef,
    pub attr: MethodAttr<TypeToken>,
    pub args: Vec<Parameter>,
    pub return_type: TypeToken,
    pub call_convention: CallConvention,

    pub generic_bounds: Option<Vec<GenericBounds>>,

    pub instructions: Vec<BinaryInstruction>,
}

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Parameter {
    pub ty: TypeToken,
    pub attr: ParameterAttr,
}

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct MethodSpec {
    pub m: u32,
    pub generics: Vec<TypeToken>,
}
