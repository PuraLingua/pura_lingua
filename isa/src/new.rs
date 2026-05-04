use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};

use global_proc_macros::{Transpose, WithType};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr, display_args};

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection, Transpose)]
#[transpose(TTypeRef, TMethodRef)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    NewObject {
        ty: TTypeRef,
        ctor_name: TMethodRef,
        args: Vec<TRegisterAddr>,
        output: TRegisterAddr,
    },
    NewArray {
        element_type: TTypeRef,
        len: u64,
        output: TRegisterAddr,
    },
    NewDynamicArray {
        element_type: TTypeRef,
        len_addr: TRegisterAddr,
        output: TRegisterAddr,
    },
}

impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    #[allow(nonstandard_style)]
    pub fn map<
        __TTypeRef,
        __TMethodRef,
        __TRegisterAddr,
        __F_TTypeRef,
        __F_TMethodRef,
        __F_TRegisterAddr,
    >(
        self,
        mut f_TTypeRef: __F_TTypeRef,
        mut f_TMethodRef: __F_TMethodRef,
        mut f_TRegisterAddr: __F_TRegisterAddr,
    ) -> Instruction_New<__TTypeRef, __TMethodRef, __TRegisterAddr>
    where
        __TRegisterAddr: IRegisterAddr,
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        __F_TMethodRef: ::core::ops::FnMut(TMethodRef) -> __TMethodRef,
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
    {
        match self {
            Instruction_New::NewObject {
                ty,
                ctor_name,
                args,
                output,
            } => Instruction_New::NewObject {
                ty: f_TTypeRef(ty),
                ctor_name: f_TMethodRef(ctor_name),
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                output: f_TRegisterAddr(output),
            },
            Instruction_New::NewArray {
                element_type,
                len,
                output,
            } => Instruction_New::NewArray {
                element_type: f_TTypeRef(element_type),
                len: len,
                output: f_TRegisterAddr(output),
            },
            Instruction_New::NewDynamicArray {
                element_type,
                len_addr,
                output,
            } => Instruction_New::NewDynamicArray {
                element_type: f_TTypeRef(element_type),
                len_addr: f_TRegisterAddr(len_addr),
                output: f_TRegisterAddr(output),
            },
        }
    }
}

impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_type_ref<__TTypeRef, __F_TTypeRef>(
        self,
        f: __F_TTypeRef,
    ) -> Instruction_New<__TTypeRef, TMethodRef, TRegisterAddr>
    where
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(f, core::convert::identity, core::convert::identity)
    }
}
impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_method_ref<__TMethodRef, __F_TMethodRef>(
        self,
        f: __F_TMethodRef,
    ) -> Instruction_New<TTypeRef, __TMethodRef, TRegisterAddr>
    where
        __F_TMethodRef: ::core::ops::FnMut(TMethodRef) -> __TMethodRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(core::convert::identity, f, core::convert::identity)
    }
}
impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_register_addr<__TRegisterAddr, __F_TRegisterAddr>(
        self,
        f: __F_TRegisterAddr,
    ) -> Instruction_New<TTypeRef, TMethodRef, __TRegisterAddr>
    where
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
        __TRegisterAddr: IRegisterAddr,
    {
        self.map(core::convert::identity, core::convert::identity, f)
    }
}

impl<TTypeRef, TMethodRef> Instruction_New<TTypeRef, TMethodRef, RegisterAddr> {
    pub fn try_into_short(
        self,
    ) -> Result<Instruction_New<TTypeRef, TMethodRef, ShortRegisterAddr>, Self> {
        match self {
            Instruction_New::NewObject {
                ty,
                ctor_name,
                args,
                output,
            } => match args
                .iter()
                .copied()
                .map(RegisterAddr::try_into_short)
                .try_collect::<Vec<_>>()
                .and_then(|args| output.try_into_short().map(|output| (args, output)))
            {
                Some((args, output)) => Ok(Instruction_New::NewObject {
                    ty,
                    ctor_name,
                    args,
                    output,
                }),
                None => Err(Instruction_New::NewObject {
                    ty,
                    ctor_name,
                    args,
                    output,
                }),
            },
            Instruction_New::NewArray {
                element_type,
                len,
                output,
            } => match output.try_into_short() {
                Some(output) => Ok(Instruction_New::NewArray {
                    element_type,
                    len,
                    output,
                }),
                None => Err(Instruction_New::NewArray {
                    element_type,
                    len,
                    output,
                }),
            },
            Instruction_New::NewDynamicArray {
                element_type,
                len_addr,
                output,
            } => match len_addr
                .try_into_short()
                .and_then(|len_addr| output.try_into_short().map(|output| (len_addr, output)))
            {
                Some((len_addr, output)) => Ok(Instruction_New::NewDynamicArray {
                    element_type,
                    len_addr,
                    output,
                }),
                None => Err(Instruction_New::NewDynamicArray {
                    element_type,
                    len_addr,
                    output,
                }),
            },
        }
    }
}

impl<TTypeRef, TMethodRef, TRegisterAddr: IRegisterAddr> Display
    for Instruction_New<TTypeRef, TMethodRef, TRegisterAddr>
where
    TTypeRef: Display,
    TMethodRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_New::NewObject {
                ty,
                ctor_name,
                args,
                output,
            } => f.write_fmt(format_args!(
                " Object {ty} {ctor_name}({}) -> {output:#x}",
                display_args(args)
            )),
            Instruction_New::NewArray {
                element_type,
                len,
                output,
            } => f.write_fmt(format_args!(
                " Array[{element_type}] of length {len}({len:#x}) -> {output:#x}"
            )),
            Instruction_New::NewDynamicArray {
                element_type,
                len_addr,
                output,
            } => f.write_fmt(format_args!(
                " DynamicArray[{element_type}] of length {len_addr:#x} -> {output:#x}"
            )),
        }
    }
}
