use std::range::Range;

use binary_core::traits::StringRef;
use global::{
    attrs::{CallConvention, MethodAttr, ParameterAttr},
    instruction::Instruction,
};
use proc_macros::{ReadFromSection, WriteToSection};

use crate::{
    item_token::{MethodToken, TypeToken},
    ty::GenericCountRequirement,
};

use super::GenericBounds;

pub type BinaryInstruction = Instruction<StringRef, TypeToken, MethodToken, u32>;

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct Method {
    pub name: StringRef,
    pub attr: MethodAttr<TypeToken>,
    pub generic_count_requirement: GenericCountRequirement,
    pub args: Vec<Parameter>,
    pub return_type: TypeToken,
    pub call_convention: CallConvention,

    pub generic_bounds: Option<Vec<GenericBounds>>,

    pub instructions: Vec<BinaryInstruction>,

    pub exception_table: Vec<ExceptionTableEntry>,
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

#[derive(Clone, Debug, ReadFromSection, WriteToSection)]
pub struct ExceptionTableEntry {
    pub range: Range<u64>,
    pub exception_type: TypeToken,
    /// Sign should be `static ([!]System::Exception) -> [!]System::Boolean`
    pub filter: Option<(TypeToken, MethodToken)>,
    pub catch: Range<u64>,
    pub finally: Option<Range<u64>>,
    pub fault: Option<Range<u64>>,
}
