use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::WithType;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, WithType, ReadFromSection, WriteToSection, derive_more::From)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Calculate<TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    U8(Instruction_UntypedCalculate<TRegisterAddr, u8>),
    U16(Instruction_UntypedCalculate<TRegisterAddr, u16>),
    U32(Instruction_UntypedCalculate<TRegisterAddr, u32>),
    U64(Instruction_UntypedCalculate<TRegisterAddr, u64>),

    I8(Instruction_UntypedCalculate<TRegisterAddr, i8>),
    I16(Instruction_UntypedCalculate<TRegisterAddr, i16>),
    I32(Instruction_UntypedCalculate<TRegisterAddr, i32>),
    I64(Instruction_UntypedCalculate<TRegisterAddr, i64>),
}

#[allow(nonstandard_style)]
impl<TRegisterAddr: IRegisterAddr> Instruction_Calculate<TRegisterAddr> {
    #[inline(always)]
    pub const fn USize(ins: Instruction_UntypedCalculate<TRegisterAddr, usize>) -> Self {
        cfg_select! {
            target_pointer_width = "16" => {
                Self::U16(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
            target_pointer_width = "32" => {
                Self::U32(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
            target_pointer_width = "64" => {
                Self::U64(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
        }
    }
    #[inline(always)]
    pub const fn ISize(ins: Instruction_UntypedCalculate<TRegisterAddr, isize>) -> Self {
        cfg_select! {
            target_pointer_width = "16" => {
                Self::I16(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
            target_pointer_width = "32" => {
                Self::I32(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
            target_pointer_width = "64" => {
                Self::I64(unsafe { std::intrinsics::transmute_unchecked(ins) })
            }
        }
    }
}

impl Instruction_Calculate<RegisterAddr> {
    pub fn try_into_short(self) -> Result<Instruction_Calculate<ShortRegisterAddr>, Self> {
        match self {
            Instruction_Calculate::U8(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::U8)
                .map_err(Self::U8),
            Instruction_Calculate::U16(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::U16)
                .map_err(Self::U16),
            Instruction_Calculate::U32(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::U32)
                .map_err(Self::U32),
            Instruction_Calculate::U64(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::U64)
                .map_err(Self::U64),

            Instruction_Calculate::I8(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::I8)
                .map_err(Self::I8),
            Instruction_Calculate::I16(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::I16)
                .map_err(Self::I16),
            Instruction_Calculate::I32(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::I32)
                .map_err(Self::I32),
            Instruction_Calculate::I64(ins) => ins
                .try_into_short()
                .map(Instruction_Calculate::I64)
                .map_err(Self::I64),
        }
    }
}

impl<TRegisterAddr: IRegisterAddr> Display for Instruction_Calculate<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_Calculate::U8(ins) => f.write_fmt(format_args!("U8{ins}")),
            Instruction_Calculate::U16(ins) => f.write_fmt(format_args!("U16{ins}")),
            Instruction_Calculate::U32(ins) => f.write_fmt(format_args!("U32{ins}")),
            Instruction_Calculate::U64(ins) => f.write_fmt(format_args!("U64{ins}")),

            Instruction_Calculate::I8(ins) => f.write_fmt(format_args!("I8{ins}")),
            Instruction_Calculate::I16(ins) => f.write_fmt(format_args!("I16{ins}")),
            Instruction_Calculate::I32(ins) => f.write_fmt(format_args!("I32{ins}")),
            Instruction_Calculate::I64(ins) => f.write_fmt(format_args!("I64{ins}")),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_UntypedCalculate<TRegisterAddr, TRust: Copy>
where
    TRegisterAddr: IRegisterAddr,
{
    Add {
        lhs: TRegisterAddr,
        rhs: TRegisterAddr,
        target: TRegisterAddr,
    },
    Sub {
        lhs: TRegisterAddr,
        rhs: TRegisterAddr,
        target: TRegisterAddr,
    },
    Mul {
        lhs: TRegisterAddr,
        rhs: TRegisterAddr,
        target: TRegisterAddr,
    },
    Div {
        lhs: TRegisterAddr,
        rhs: TRegisterAddr,
        target: TRegisterAddr,
    },
    Rem {
        lhs: TRegisterAddr,
        rhs: TRegisterAddr,
        target: TRegisterAddr,
    },

    ConstAddTo {
        target: TRegisterAddr,
        data: TRust,
    },
    ConstSubTo {
        target: TRegisterAddr,
        data: TRust,
    },
    ConstMulTo {
        target: TRegisterAddr,
        data: TRust,
    },
    ConstDivTo {
        target: TRegisterAddr,
        data: TRust,
    },
    ConstRemTo {
        target: TRegisterAddr,
        data: TRust,
    },

    SubByConst {
        target: TRegisterAddr,
        data: TRust,
    },
    DivByConst {
        target: TRegisterAddr,
        data: TRust,
    },
    RemByConst {
        target: TRegisterAddr,
        data: TRust,
    },

    AddOne {
        target: TRegisterAddr,
    },
    SubOne {
        target: TRegisterAddr,
    },
}

impl<TRust: Copy> Instruction_UntypedCalculate<RegisterAddr, TRust> {
    pub fn try_into_short(
        self,
    ) -> Result<Instruction_UntypedCalculate<ShortRegisterAddr, TRust>, Self> {
        use Instruction_UntypedCalculate::*;

        match self {
            Add { lhs, rhs, target } => lhs
                .try_into_short()
                .and_then(|lhs| rhs.try_into_short().map(|rhs| (lhs, rhs)))
                .and_then(|(lhs, rhs)| {
                    target
                        .try_into_short()
                        .map(|target| Add { lhs, rhs, target })
                })
                .ok_or(self),
            Sub { lhs, rhs, target } => lhs
                .try_into_short()
                .and_then(|lhs| rhs.try_into_short().map(|rhs| (lhs, rhs)))
                .and_then(|(lhs, rhs)| {
                    target
                        .try_into_short()
                        .map(|target| Sub { lhs, rhs, target })
                })
                .ok_or(self),
            Mul { lhs, rhs, target } => lhs
                .try_into_short()
                .and_then(|lhs| rhs.try_into_short().map(|rhs| (lhs, rhs)))
                .and_then(|(lhs, rhs)| {
                    target
                        .try_into_short()
                        .map(|target| Mul { lhs, rhs, target })
                })
                .ok_or(self),
            Div { lhs, rhs, target } => lhs
                .try_into_short()
                .and_then(|lhs| rhs.try_into_short().map(|rhs| (lhs, rhs)))
                .and_then(|(lhs, rhs)| {
                    target
                        .try_into_short()
                        .map(|target| Div { lhs, rhs, target })
                })
                .ok_or(self),
            Rem { lhs, rhs, target } => lhs
                .try_into_short()
                .and_then(|lhs| rhs.try_into_short().map(|rhs| (lhs, rhs)))
                .and_then(|(lhs, rhs)| {
                    target
                        .try_into_short()
                        .map(|target| Rem { lhs, rhs, target })
                })
                .ok_or(self),

            ConstAddTo { target, data } => target
                .try_into_short()
                .map(|target| ConstAddTo { target, data })
                .ok_or(self),
            ConstSubTo { target, data } => target
                .try_into_short()
                .map(|target| ConstSubTo { target, data })
                .ok_or(self),
            ConstMulTo { target, data } => target
                .try_into_short()
                .map(|target| ConstMulTo { target, data })
                .ok_or(self),
            ConstDivTo { target, data } => target
                .try_into_short()
                .map(|target| ConstDivTo { target, data })
                .ok_or(self),
            ConstRemTo { target, data } => target
                .try_into_short()
                .map(|target| ConstRemTo { target, data })
                .ok_or(self),

            SubByConst { target, data } => target
                .try_into_short()
                .map(|target| ConstSubTo { target, data })
                .ok_or(self),
            DivByConst { target, data } => target
                .try_into_short()
                .map(|target| ConstDivTo { target, data })
                .ok_or(self),
            RemByConst { target, data } => target
                .try_into_short()
                .map(|target| ConstRemTo { target, data })
                .ok_or(self),

            AddOne { target } => target
                .try_into_short()
                .map(|target| AddOne { target })
                .ok_or(self),
            SubOne { target } => target
                .try_into_short()
                .map(|target| SubOne { target })
                .ok_or(self),
        }
    }
}

impl<TRegisterAddr: IRegisterAddr, TRust: Display + std::fmt::LowerHex + Copy> Display
    for Instruction_UntypedCalculate<TRegisterAddr, TRust>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_UntypedCalculate::Add { lhs, rhs, target } => {
                f.write_fmt(format_args!(" {lhs:#x} + {rhs:#x} -> {target:#x}"))
            }
            Instruction_UntypedCalculate::Sub { lhs, rhs, target } => {
                f.write_fmt(format_args!(" {lhs:#x} - {rhs:#x} -> {target:#x}"))
            }
            Instruction_UntypedCalculate::Mul { lhs, rhs, target } => {
                f.write_fmt(format_args!(" {lhs:#x} * {rhs:#x} -> {target:#x}"))
            }
            Instruction_UntypedCalculate::Div { lhs, rhs, target } => {
                f.write_fmt(format_args!(" {lhs:#x} / {rhs:#x} -> {target:#x}"))
            }
            Instruction_UntypedCalculate::Rem { lhs, rhs, target } => {
                f.write_fmt(format_args!(" {lhs:#x} % {rhs:#x} -> {target:#x}"))
            }

            Instruction_UntypedCalculate::ConstAddTo { target, data } => {
                write!(f, " {target:#x} + const{data}(0x{data:#x}) -> {target:#x}")
            }
            Instruction_UntypedCalculate::ConstSubTo { target, data } => {
                write!(f, " {target:#x} - const{data}(0x{data:#x}) -> {target:#x}")
            }
            Instruction_UntypedCalculate::ConstMulTo { target, data } => {
                write!(f, " {target:#x} * const{data}(0x{data:#x}) -> {target:#x}")
            }
            Instruction_UntypedCalculate::ConstDivTo { target, data } => {
                write!(f, " {target:#x} / const{data}(0x{data:#x}) -> {target:#x}")
            }
            Instruction_UntypedCalculate::ConstRemTo { target, data } => {
                write!(f, " {target:#x} % const{data}(0x{data:#x}) -> {target:#x}")
            }

            Instruction_UntypedCalculate::SubByConst { target, data } => {
                write!(f, " const{data}(0x{data:#x}) - {target:#x} -> {target:#x}")
            }
            Instruction_UntypedCalculate::DivByConst { target, data } => {
                write!(f, " const{data}(0x{data:#x}) / {target:#x} -> {target:#x}")
            }
            Instruction_UntypedCalculate::RemByConst { target, data } => {
                write!(f, " const{data}(0x{data:#x}) % {target:#x} -> {target:#x}")
            }

            Instruction_UntypedCalculate::AddOne { target } => {
                write!(f, " {target} + const1 -> {target:#x}")
            }
            Instruction_UntypedCalculate::SubOne { target } => {
                write!(f, " {target} - const1 -> {target:#x}")
            }
        }
    }
}
