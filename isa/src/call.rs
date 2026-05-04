use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use binary_proc_macros::{ReadFromSection, WriteToSection};

use global_proc_macros::{Transpose, WithType};
use non_purus_call::NonPurusCallConfiguration;

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr, display_args};

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection, Transpose)]
#[transpose(TTypeRef, TMethodRef)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
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
    InterfaceCall {
        interface: TTypeRef,
        val: TRegisterAddr,
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

impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
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
    ) -> Instruction_Call<__TTypeRef, __TMethodRef, __TRegisterAddr>
    where
        __TRegisterAddr: IRegisterAddr,
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        __F_TMethodRef: ::core::ops::FnMut(TMethodRef) -> __TMethodRef,
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
    {
        match self {
            Instruction_Call::InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => Instruction_Call::InstanceCall {
                val: f_TRegisterAddr(val),
                method: f_TMethodRef(method),
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                ret_at: f_TRegisterAddr(ret_at),
            },
            Instruction_Call::StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => Instruction_Call::StaticCall {
                ty: f_TTypeRef(ty),
                method: f_TMethodRef(method),
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                ret_at: f_TRegisterAddr(ret_at),
            },
            Instruction_Call::InterfaceCall {
                interface,
                val,
                method,
                args,
                ret_at,
            } => Instruction_Call::InterfaceCall {
                interface: f_TTypeRef(interface),
                val: f_TRegisterAddr(val),
                method: f_TMethodRef(method),
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                ret_at: f_TRegisterAddr(ret_at),
            },
            Instruction_Call::StaticNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => Instruction_Call::StaticNonPurusCall {
                f_pointer: f_TRegisterAddr(f_pointer),
                config: config,
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                ret_at: f_TRegisterAddr(ret_at),
            },
            Instruction_Call::DynamicNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => Instruction_Call::DynamicNonPurusCall {
                f_pointer: f_TRegisterAddr(f_pointer),
                config: f_TRegisterAddr(config),
                args: args.into_iter().map(&mut f_TRegisterAddr).collect(),
                ret_at: f_TRegisterAddr(ret_at),
            },
        }
    }
}

impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_type_ref<__TTypeRef, __F_TTypeRef>(
        self,
        f: __F_TTypeRef,
    ) -> Instruction_Call<__TTypeRef, TMethodRef, TRegisterAddr>
    where
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(f, core::convert::identity, core::convert::identity)
    }
}
impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_method_ref<__TMethodRef, __F_TMethodRef>(
        self,
        f: __F_TMethodRef,
    ) -> Instruction_Call<TTypeRef, __TMethodRef, TRegisterAddr>
    where
        __F_TMethodRef: ::core::ops::FnMut(TMethodRef) -> __TMethodRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(core::convert::identity, f, core::convert::identity)
    }
}
impl<TTypeRef, TMethodRef, TRegisterAddr> Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_register_addr<__TRegisterAddr, __F_TRegisterAddr>(
        self,
        f: __F_TRegisterAddr,
    ) -> Instruction_Call<TTypeRef, TMethodRef, __TRegisterAddr>
    where
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
        __TRegisterAddr: IRegisterAddr,
    {
        self.map(core::convert::identity, core::convert::identity, f)
    }
}

impl<TTypeRef, TMethodRef> Instruction_Call<TTypeRef, TMethodRef, RegisterAddr> {
    pub fn try_into_short(
        self,
    ) -> Result<Instruction_Call<TTypeRef, TMethodRef, ShortRegisterAddr>, Self> {
        match self {
            Instruction_Call::InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => match val.try_into_short().and_then(|val| {
                args.iter()
                    .copied()
                    .map(RegisterAddr::try_into_short)
                    .try_collect::<Vec<_>>()
                    .and_then(|args| ret_at.try_into_short().map(|ret_at| (args, ret_at)))
                    .map(|(args, ret_at)| (val, args, ret_at))
            }) {
                Some((val, args, ret_at)) => Ok(Instruction_Call::InstanceCall {
                    val,
                    method,
                    args,
                    ret_at,
                }),
                None => Err(Instruction_Call::InstanceCall {
                    val,
                    method,
                    args,
                    ret_at,
                }),
            },
            Instruction_Call::StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => match args
                .iter()
                .copied()
                .map(RegisterAddr::try_into_short)
                .try_collect::<Vec<_>>()
                .and_then(|args| ret_at.try_into_short().map(|ret_at| (args, ret_at)))
            {
                Some((args, ret_at)) => Ok(Instruction_Call::StaticCall {
                    ty,
                    method,
                    args,
                    ret_at,
                }),
                None => Err(Instruction_Call::StaticCall {
                    ty,
                    method,
                    args,
                    ret_at,
                }),
            },
            Instruction_Call::InterfaceCall {
                interface,
                val,
                method,
                args,
                ret_at,
            } => match val.try_into_short().and_then(|val| {
                args.iter()
                    .copied()
                    .map(RegisterAddr::try_into_short)
                    .try_collect::<Vec<_>>()
                    .map(|args| (val, args))
                    .and_then(|(val, args)| {
                        ret_at.try_into_short().map(|ret_at| (val, args, ret_at))
                    })
            }) {
                Some((val, args, ret_at)) => Ok(Instruction_Call::InterfaceCall {
                    interface,
                    val,
                    method,
                    args,
                    ret_at,
                }),
                None => Err(Instruction_Call::InterfaceCall {
                    interface,
                    val,
                    method,
                    args,
                    ret_at,
                }),
            },
            Instruction_Call::StaticNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => match f_pointer.try_into_short().and_then(|f_pointer| {
                args.iter()
                    .copied()
                    .map(RegisterAddr::try_into_short)
                    .try_collect::<Vec<_>>()
                    .and_then(|args| ret_at.try_into_short().map(|ret_at| (args, ret_at)))
                    .map(|(args, ret_at)| (f_pointer, args, ret_at))
            }) {
                Some((f_pointer, args, ret_at)) => Ok(Instruction_Call::StaticNonPurusCall {
                    f_pointer,
                    config,
                    args,
                    ret_at,
                }),
                None => Err(Instruction_Call::StaticNonPurusCall {
                    f_pointer,
                    config,
                    args,
                    ret_at,
                }),
            },
            Instruction_Call::DynamicNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => match f_pointer.try_into_short().and_then(|f_pointer| {
                config.try_into_short().and_then(|config| {
                    args.iter()
                        .copied()
                        .map(RegisterAddr::try_into_short)
                        .try_collect::<Vec<_>>()
                        .and_then(|args| ret_at.try_into_short().map(|ret_at| (args, ret_at)))
                        .map(|(args, ret_at)| (f_pointer, config, args, ret_at))
                })
            }) {
                Some((f_pointer, config, args, ret_at)) => {
                    Ok(Instruction_Call::DynamicNonPurusCall {
                        f_pointer,
                        config,
                        args,
                        ret_at,
                    })
                }
                None => Err(Instruction_Call::DynamicNonPurusCall {
                    f_pointer,
                    config,
                    args,
                    ret_at,
                }),
            },
        }
    }
}

impl<TTypeRef, TMethodRef, TRegisterAddr: IRegisterAddr> Display
    for Instruction_Call<TTypeRef, TMethodRef, TRegisterAddr>
where
    TTypeRef: Display,
    TMethodRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_Call::InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => f.write_fmt(format_args!(
                " InstanceCall {val:#x} {method}({}) -> {ret_at:#x}",
                display_args(args)
            )),
            Instruction_Call::StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => f.write_fmt(format_args!(
                " StaticCall {ty} {method}({}) -> {ret_at:#x}",
                display_args(args)
            )),
            Instruction_Call::InterfaceCall {
                interface,
                val,
                method,
                args,
                ret_at,
            } => f.write_fmt(format_args!(
                " InterfaceCall {val:#x} as {interface} {method}({}) -> {ret_at:#x}",
                display_args(args)
            )),
            Instruction_Call::StaticNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => f.write_fmt(format_args!(
                " StaticNonPurusCall #{config:?} {f_pointer:#x}({}) -> {ret_at:#x}",
                display_args(args)
            )),
            Instruction_Call::DynamicNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => f.write_fmt(format_args!(
                " StaticNonPurusCall #{config:#x} {f_pointer:#x}({}) -> {ret_at:#x}",
                display_args(args)
            )),
        }
    }
}
