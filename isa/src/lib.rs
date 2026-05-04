#![allow(non_camel_case_types, internal_features)]
#![feature(iterator_try_collect)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_cmp)]
#![feature(const_default)]
#![feature(unboxed_closures)]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(fn_traits)]
#![feature(decl_macro)]
#![feature(const_try)]
#![feature(const_convert)]
#![feature(const_option_ops)]
#![feature(const_destruct)]
#![feature(const_result_trait_fn)]

use std::fmt::Display;
use std::ptr::NonNull;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::WithType;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

mod jumping;
pub use jumping::*;

mod register_addr;
pub use register_addr::*;

mod calculate;
mod call;
mod check;
mod jump;
mod load;
mod new;
mod read_write_pointer;
mod set;

pub use calculate::*;
pub use call::*;
pub use check::*;
pub use jump::*;
pub use load::*;
pub use new::*;
pub use read_write_pointer::*;
pub use set::*;

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction<TString, TTypeRef, TMethodRef, TFieldRef> {
    Nop,
    Load(Instruction_Load<TString, TTypeRef, TFieldRef, RegisterAddr>),
    SLoad(Instruction_Load<TString, TTypeRef, TFieldRef, ShortRegisterAddr>),

    ReadPointerTo(ReadPointerTo),
    SReadPointerTo(SReadPointerTo),

    WritePointer(WritePointer),
    SWritePointer(SWritePointer),

    Check(Instruction_Check),
    SCheck(Instruction_SCheck),

    New(Instruction_New<TTypeRef, TMethodRef, RegisterAddr>),
    SNew(Instruction_New<TTypeRef, TMethodRef, ShortRegisterAddr>),

    Call(Instruction_Call<TTypeRef, TMethodRef, RegisterAddr>),
    SCall(Instruction_Call<TTypeRef, TMethodRef, ShortRegisterAddr>),

    Set(Instruction_Set<TTypeRef, TFieldRef, RegisterAddr>),
    SSet(Instruction_Set<TTypeRef, TFieldRef, ShortRegisterAddr>),

    Calculate(Instruction_Calculate<RegisterAddr>),
    SCalculate(Instruction_Calculate<ShortRegisterAddr>),

    Throw { exception_addr: RegisterAddr },
    SThrow { exception_addr: ShortRegisterAddr },
    Rethrow,

    ReturnVal { register_addr: RegisterAddr },
    SReturnVal { register_addr: ShortRegisterAddr },

    Jump(Instruction_Jump<RegisterAddr>),
    SJump(Instruction_Jump<ShortRegisterAddr>),
}

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
{
    pub fn type_ptr(&self) -> NonNull<u8> {
        NonNull::from_ref(self).cast()
    }
    pub fn data_ptr(&self) -> NonNull<()> {
        unsafe {
            NonNull::from_ref(self)
                .cast::<()>()
                .byte_add(size_of::<u8>())
        }
    }
}

// cSpell:disable

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<Option<TString>, Option<TTypeRef>, Option<TMethodRef>, Option<TFieldRef>>
{
    pub fn transpose(self) -> Option<Instruction<TString, TTypeRef, TMethodRef, TFieldRef>> {
        use Instruction::*;
        match self {
            Nop => Some(Nop),
            Load(ins) => ins.transpose().map(Load),
            SLoad(ins) => ins.transpose().map(SLoad),

            ReadPointerTo(ins) => Some(ReadPointerTo(ins)),
            SReadPointerTo(ins) => Some(SReadPointerTo(ins)),

            WritePointer(ins) => Some(WritePointer(ins)),
            SWritePointer(ins) => Some(SWritePointer(ins)),

            Check(ins) => Some(Check(ins)),
            SCheck(ins) => Some(SCheck(ins)),

            New(ins) => ins.transpose().map(New),
            SNew(ins) => ins.transpose().map(SNew),

            Call(ins) => ins.transpose().map(Call),
            SCall(ins) => ins.transpose().map(SCall),

            Set(ins) => ins.transpose().map(Set),
            SSet(ins) => ins.transpose().map(SSet),

            Calculate(ins) => Some(Calculate(ins)),
            SCalculate(ins) => Some(SCalculate(ins)),

            Throw { exception_addr } => Some(Throw { exception_addr }),
            SThrow { exception_addr } => Some(SThrow { exception_addr }),
            Rethrow => Some(Rethrow),

            ReturnVal { register_addr } => Some(ReturnVal { register_addr }),
            SReturnVal { register_addr } => Some(SReturnVal { register_addr }),

            Jump(ins) => Some(Jump(ins)),
            SJump(ins) => Some(SJump(ins)),
        }
    }
}

impl<TString, E1, TTypeRef, E2, TMethodRef, E3, TFieldRef, E4>
    Instruction<
        Result<TString, E1>,
        Result<TTypeRef, E2>,
        Result<TMethodRef, E3>,
        Result<TFieldRef, E4>,
    >
{
    /// Return the first error if there are many errors
    pub fn transpose<UniE>(
        self,
    ) -> Result<Instruction<TString, TTypeRef, TMethodRef, TFieldRef>, UniE>
    where
        UniE: From<E1> + From<E2> + From<E3> + From<E4>,
    {
        use Instruction::*;
        match self {
            Nop => Ok(Nop),
            Load(ins) => ins.transpose().map(Load),
            SLoad(ins) => ins.transpose().map(SLoad),

            ReadPointerTo(ins) => Ok(ReadPointerTo(ins)),
            SReadPointerTo(ins) => Ok(SReadPointerTo(ins)),

            WritePointer(ins) => Ok(WritePointer(ins)),
            SWritePointer(ins) => Ok(SWritePointer(ins)),

            Check(ins) => Ok(Check(ins)),
            SCheck(ins) => Ok(SCheck(ins)),

            New(ins) => ins.transpose().map(New),
            SNew(ins) => ins.transpose().map(SNew),

            Call(ins) => ins.transpose().map(Call),
            SCall(ins) => ins.transpose().map(SCall),

            Set(ins) => ins.transpose().map(Set),
            SSet(ins) => ins.transpose().map(SSet),

            Calculate(ins) => Ok(Calculate(ins)),
            SCalculate(ins) => Ok(SCalculate(ins)),

            Throw { exception_addr } => Ok(Throw { exception_addr }),
            SThrow { exception_addr } => Ok(SThrow { exception_addr }),
            Rethrow => Ok(Rethrow),

            ReturnVal { register_addr } => Ok(ReturnVal { register_addr }),
            SReturnVal { register_addr } => Ok(SReturnVal { register_addr }),

            Jump(ins) => Ok(Jump(ins)),
            SJump(ins) => Ok(SJump(ins)),
        }
    }
}

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
{
    #[allow(nonstandard_style)]
    pub fn map<
        _TString,
        _TTypeRef,
        _TMethodRef,
        _TFieldRef,
        FString,
        FTypeRef,
        FMethodRef,
        FFieldRef,
    >(
        self,
        f_TString: FString,
        f_TTypeRef: FTypeRef,
        f_TMethodRef: FMethodRef,
        f_TFieldRef: FFieldRef,
    ) -> Instruction<_TString, _TTypeRef, _TMethodRef, _TFieldRef>
    where
        FString: FnMut(TString) -> _TString,
        FTypeRef: FnMut(TTypeRef) -> _TTypeRef,
        FMethodRef: FnMut(TMethodRef) -> _TMethodRef,
        FFieldRef: FnMut(TFieldRef) -> _TFieldRef,
    {
        use Instruction::*;
        #[inline(always)]
        const fn noop<T>(v: T) -> T {
            v
        }
        match self {
            Nop => Nop,
            Load(ins) => Load(ins.map(f_TString, f_TTypeRef, f_TFieldRef, noop)),
            SLoad(ins) => SLoad(ins.map(f_TString, f_TTypeRef, f_TFieldRef, noop)),

            ReadPointerTo(ins) => ReadPointerTo(ins),
            SReadPointerTo(ins) => SReadPointerTo(ins),

            WritePointer(ins) => WritePointer(ins),
            SWritePointer(ins) => SWritePointer(ins),

            Check(ins) => Check(ins),
            SCheck(ins) => SCheck(ins),

            New(ins) => New(ins.map(f_TTypeRef, f_TMethodRef, noop)),
            SNew(ins) => SNew(ins.map(f_TTypeRef, f_TMethodRef, noop)),

            Call(ins) => Call(ins.map(f_TTypeRef, f_TMethodRef, noop)),
            SCall(ins) => SCall(ins.map(f_TTypeRef, f_TMethodRef, noop)),

            Set(ins) => Set(ins.map(f_TTypeRef, f_TFieldRef, noop)),
            SSet(ins) => SSet(ins.map(f_TTypeRef, f_TFieldRef, noop)),

            Calculate(ins) => Calculate(ins),
            SCalculate(ins) => SCalculate(ins),

            Throw { exception_addr } => Throw { exception_addr },
            SThrow { exception_addr } => SThrow { exception_addr },
            Rethrow => Rethrow,

            ReturnVal { register_addr } => ReturnVal { register_addr },
            SReturnVal { register_addr } => SReturnVal { register_addr },

            Jump(ins) => Jump(ins),
            SJump(ins) => SJump(ins),
        }
    }
}

fn display_args<TRegisterAddr: IRegisterAddr>(args: &[TRegisterAddr]) -> String {
    args.iter()
        .map(|x| format!("{x:#x}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl<TString, TTypeRef, TMethodRef, TFieldRef> Display
    for Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
where
    TString: Display,
    TTypeRef: Display,
    TMethodRef: Display,
    TFieldRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NAME: &str = stringify!(Instruction);
        match self {
            Instruction::Nop => f.write_fmt(format_args!("{NAME}::NOP")),

            Instruction::Load(ins) => f.write_fmt(format_args!("{NAME}::Load{ins}")),
            Instruction::SLoad(ins) => f.write_fmt(format_args!("{NAME}::SLoad{ins}")),

            Instruction::ReadPointerTo(ins) => {
                f.write_fmt(format_args!("{NAME}::ReadPointerTo{ins}"))
            }
            Instruction::SReadPointerTo(ins) => {
                f.write_fmt(format_args!("{NAME}::SReadPointerTo{ins}"))
            }

            Instruction::WritePointer(ins) => {
                f.write_fmt(format_args!("{NAME}::WritePointer{ins}"))
            }
            Instruction::SWritePointer(ins) => {
                f.write_fmt(format_args!("{NAME}::SWritePointer{ins}"))
            }

            Instruction::Check(ins) => f.write_fmt(format_args!("{NAME}::Check{ins}")),
            Instruction::SCheck(ins) => f.write_fmt(format_args!("{NAME}::SCheck{ins}")),

            Instruction::New(ins) => f.write_fmt(format_args!("{NAME}::New{ins}")),
            Instruction::SNew(ins) => f.write_fmt(format_args!("{NAME}::SNew{ins}")),

            Instruction::Call(ins) => f.write_fmt(format_args!("{NAME}::Call{ins}")),
            Instruction::SCall(ins) => f.write_fmt(format_args!("{NAME}::SCall{ins}")),

            Instruction::Set(ins) => f.write_fmt(format_args!("{NAME}::Set{ins}")),
            Instruction::SSet(ins) => f.write_fmt(format_args!("{NAME}::SSet{ins}")),

            Instruction::Calculate(ins) => f.write_fmt(format_args!("{NAME}::Caclulate{ins}")),
            Instruction::SCalculate(ins) => f.write_fmt(format_args!("{NAME}::SCaclulate{ins}")),

            Instruction::Throw { exception_addr } => {
                f.write_fmt(format_args!("{NAME}::Throw {exception_addr:#x}"))
            }
            Instruction::SThrow { exception_addr } => {
                f.write_fmt(format_args!("{NAME}::SThrow {exception_addr:#x}"))
            }
            Instruction::Rethrow => f.write_fmt(format_args!("{NAME}::Rethrow")),

            Instruction::ReturnVal { register_addr } => {
                f.write_fmt(format_args!("{NAME}::ReturnVal {register_addr:#x}"))
            }
            Instruction::SReturnVal { register_addr } => {
                f.write_fmt(format_args!("{NAME}::SReturnVal {register_addr:#x}"))
            }

            Instruction::Jump(ins) => f.write_fmt(format_args!("{NAME}::Jump{ins}")),
            Instruction::SJump(ins) => f.write_fmt(format_args!("{NAME}::SJump{ins}")),
        }
    }
}

// NOTE: you cannot split it into different modules
impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
{
    pub fn try_into_short(self) -> Self {
        use Instruction::*;
        match self {
            Nop => Nop,

            Load(ins) => match ins.try_into_short() {
                Ok(ins) => SLoad(ins),
                Err(ins) => Load(ins),
            },
            SLoad(ins) => SLoad(ins),

            ReadPointerTo(ins) => match ins.try_into_short() {
                Ok(ins) => SReadPointerTo(ins),
                Err(ins) => ReadPointerTo(ins),
            },
            SReadPointerTo(ins) => SReadPointerTo(ins),

            WritePointer(ins) => match ins.try_into_short() {
                Ok(ins) => SWritePointer(ins),
                Err(ins) => WritePointer(ins),
            },
            SWritePointer(ins) => SWritePointer(ins),

            Check(ins) => match ins.try_into_short() {
                Ok(ins) => SCheck(ins),
                Err(ins) => Check(ins),
            },
            SCheck(ins) => SCheck(ins),

            New(ins) => match ins.try_into_short() {
                Ok(ins) => SNew(ins),
                Err(ins) => New(ins),
            },
            SNew(ins) => SNew(ins),

            Call(ins) => match ins.try_into_short() {
                Ok(ins) => SCall(ins),
                Err(ins) => Call(ins),
            },
            SCall(ins) => SCall(ins),

            Set(ins) => match ins.try_into_short() {
                Ok(ins) => SSet(ins),
                Err(ins) => Set(ins),
            },
            SSet(ins) => SSet(ins),

            Calculate(ins) => match ins.try_into_short() {
                Ok(ins) => SCalculate(ins),
                Err(ins) => Calculate(ins),
            },
            SCalculate(ins) => SCalculate(ins),

            Throw { exception_addr } => match exception_addr.try_into_short() {
                Some(exception_addr) => SThrow { exception_addr },
                None => Throw { exception_addr },
            },
            SThrow { exception_addr } => SThrow { exception_addr },
            Rethrow => Rethrow,

            ReturnVal { register_addr } => match register_addr.try_into_short() {
                Some(register_addr) => SReturnVal { register_addr },
                None => ReturnVal { register_addr },
            },
            SReturnVal { register_addr } => SReturnVal { register_addr },

            Jump(ins) => {
                let Instruction_Jump { target, condition } = ins;
                match condition {
                    JumpCondition::Unconditional => SJump(Instruction_Jump {
                        target,
                        condition: JumpCondition::Unconditional,
                    }),
                    JumpCondition::If(cond) => cond
                        .try_into_short()
                        .map(|cond| {
                            Instruction::SJump(Instruction_Jump {
                                target,
                                condition: JumpCondition::If(cond),
                            })
                        })
                        .unwrap_or(Instruction::Jump(Instruction_Jump {
                            target,
                            condition: JumpCondition::If(cond),
                        })),
                    JumpCondition::IfCheckSucceeds(to_check) => {
                        to_check.try_to_short().map_or_else(
                            |to_check| {
                                Instruction::Jump(Instruction_Jump {
                                    target,
                                    condition: JumpCondition::IfCheckSucceeds(to_check),
                                })
                            },
                            |to_check| {
                                Instruction::SJump(Instruction_Jump {
                                    target,
                                    condition: JumpCondition::IfCheckSucceeds(to_check),
                                })
                            },
                        )
                    }
                    JumpCondition::IfCheckFails(to_check) => to_check.try_to_short().map_or_else(
                        |to_check| {
                            Instruction::Jump(Instruction_Jump {
                                target,
                                condition: JumpCondition::IfCheckFails(to_check),
                            })
                        },
                        |to_check| {
                            Instruction::SJump(Instruction_Jump {
                                target,
                                condition: JumpCondition::IfCheckFails(to_check),
                            })
                        },
                    ),
                }
            }
            SJump(ins) => SJump(ins),
        }
    }
}

// cSpell:enable
