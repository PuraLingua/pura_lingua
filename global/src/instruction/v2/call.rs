use binary_proc_macros::{ReadFromSection, WriteToSection};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::WithType;

use crate::{
    instruction::v2::IRegisterAddr, non_purus_call_configuration::NonPurusCallConfiguration,
};

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr: IRegisterAddr> {
    InstanceCall {
        val: TRegisterAddr,
        method: TMethodRef,
        args: Vec<TRegisterAddr>,
        ret_at: TRegisterAddr,
    },
    StaticCall {
        ty: TTypeRef,
        method: TMethodRef,
        args: Vec<TRegisterAddr>,
        ret_at: TRegisterAddr,
    },
    StaticNonPurusCall {
        f_pointer: TRegisterAddr,
        config: NonPurusCallConfiguration,
        args: Vec<TRegisterAddr>,
        ret_at: TRegisterAddr,
    },
    DynamicNonPurusCall {
        f_pointer: TRegisterAddr,
        config: TRegisterAddr,
        args: Vec<TRegisterAddr>,
        ret_at: TRegisterAddr,
    },
}
