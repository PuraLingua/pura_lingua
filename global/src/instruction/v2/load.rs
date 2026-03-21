use binary_proc_macros::{ReadFromSection, WriteToSection};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::{
    instruction::v2::{IRegisterAddr, RegisterAddr, ShortRegisterAddr},
    non_purus_call_configuration::NonPurusCallConfiguration,
};

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_Load<TString, TTypeRef, TFieldRef> {
    pub addr: RegisterAddr,
    pub content: LoadContent<TString, TTypeRef, TFieldRef, RegisterAddr>,
}

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_SLoad<TString, TTypeRef, TFieldRef> {
    pub addr: ShortRegisterAddr,
    pub content: LoadContent<TString, TTypeRef, TFieldRef, ShortRegisterAddr>,
}

#[repr(u32)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum LoadContent<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> {
    True,
    False,

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    This,

    String(TString),
    TypeValueSize(TTypeRef),

    NonPurusCallConfiguration(NonPurusCallConfiguration),

    Arg(u64),

    Static {
        ty: TTypeRef,
        field: TFieldRef,
    },
    Field {
        container: TRegisterAddr,
        field: TFieldRef,
    },
}
