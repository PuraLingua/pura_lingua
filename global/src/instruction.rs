#![allow(non_camel_case_types)]

use std::fmt::Display;
use std::marker::Destruct;
use std::ptr::NonNull;

use binary_proc_macros::ReadFromFile;
use binary_proc_macros::WriteToFile;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use proc_macros::WithType;

use crate::StringName;

#[repr(u64)]
#[derive(Debug, Clone, WithType, ReadFromFile, WriteToFile)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromFile, WriteToFile))]
pub enum Instruction<TTypeRef, TMethodRef, TFieldRef> {
    LoadTrue {
        register_addr: u64,
    },
    LoadFalse {
        register_addr: u64,
    },

    Load_u8 {
        register_addr: u64,
        val: u8,
    },
    Load_u64 {
        register_addr: u64,
        val: u64,
    },

    LoadThis {
        register_addr: u64,
    },

    Load_String {
        register_addr: u64,
        val: StringName,
    },

    NewObject {
        ty: TTypeRef,
        ctor_name: TMethodRef,
        args: Vec<u64>,
        register_addr: u64,
    },

    InstanceCall {
        val: u64,
        method: TMethodRef,
        args: Vec<u64>,
        ret_at: u64,
    },

    StaticCall {
        ty: TTypeRef,
        method: TMethodRef,
        args: Vec<u64>,
        ret_at: u64,
    },

    LoadArg {
        register_addr: u64,
        arg: u64,
    },

    LoadStatic {
        register_addr: u64,
        ty: TTypeRef,
        field: TFieldRef,
    },

    SetThisField {
        val_addr: u64,
        field: TFieldRef,
    },

    SetStaticField {
        val_addr: u64,
        ty: TTypeRef,
        field: TFieldRef,
    },

    Throw {
        exception_addr: u64,
    },

    ReturnVal {
        register_addr: u64,
    },
}

impl<TTypeRef, TMethodRef, TFieldRef> Instruction<TTypeRef, TMethodRef, TFieldRef> {
    pub fn type_ptr(&self) -> NonNull<u8> {
        NonNull::from_ref(self).cast()
    }
    pub fn data_ptr(&self) -> NonNull<()> {
        unsafe {
            NonNull::from_ref(self)
                .cast::<()>()
                .byte_add(size_of::<u64>())
        }
    }
}

impl<TTypeRef, TMethodRef, TFieldRef> Instruction<TTypeRef, TMethodRef, TFieldRef> {
    pub const fn map_types<_TTypeRef, _TMethodRef, _TFieldRef, FTypeRef, FMethodRef, FFieldRef>(
        self,
        f_type: FTypeRef,
        f_method: FMethodRef,
        f_field: FFieldRef,
    ) -> Instruction<_TTypeRef, _TMethodRef, _TFieldRef>
    where
        FTypeRef: [const] Fn(TTypeRef) -> _TTypeRef + [const] Destruct,
        FMethodRef: [const] Fn(TMethodRef) -> _TMethodRef + [const] Destruct,
        FFieldRef: [const] Fn(TFieldRef) -> _TFieldRef + [const] Destruct,
        Self: [const] Destruct,
    {
        use Instruction::*;
        match self {
            LoadTrue { register_addr } => LoadTrue { register_addr },
            LoadFalse { register_addr } => LoadFalse { register_addr },
            Load_u8 { register_addr, val } => Load_u8 { register_addr, val },
            Load_u64 { register_addr, val } => Load_u64 { register_addr, val },
            LoadThis { register_addr } => LoadThis { register_addr },
            Load_String { register_addr, val } => Load_String { register_addr, val },
            NewObject {
                ty,
                ctor_name,
                args,
                register_addr,
            } => NewObject {
                ty: f_type(ty),
                ctor_name: f_method(ctor_name),
                args,
                register_addr,
            },
            InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => InstanceCall {
                val,
                method: f_method(method),
                args,
                ret_at,
            },
            StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => StaticCall {
                ty: f_type(ty),
                method: f_method(method),
                args,
                ret_at,
            },
            LoadArg { register_addr, arg } => LoadArg { register_addr, arg },
            LoadStatic {
                register_addr,
                ty,
                field,
            } => LoadStatic {
                register_addr,
                ty: f_type(ty),
                field: f_field(field),
            },
            SetThisField { val_addr, field } => SetThisField {
                val_addr,
                field: f_field(field),
            },
            SetStaticField {
                val_addr,
                ty,
                field,
            } => SetStaticField {
                val_addr,
                ty: f_type(ty),
                field: f_field(field),
            },
            Throw { exception_addr } => Throw { exception_addr },
            ReturnVal { register_addr } => ReturnVal { register_addr },
        }
    }
}

impl<TTypeRef, TMethodRef, TFieldRef> Display for Instruction<TTypeRef, TMethodRef, TFieldRef>
where
    TTypeRef: Display,
    TMethodRef: Display,
    TFieldRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NAME: &str = "Instruction";
        match self {
            Instruction::LoadTrue { register_addr } => {
                f.write_fmt(const_format_args!("{NAME}::LoadTrue {register_addr:#x}"))
            }
            Instruction::LoadFalse { register_addr } => {
                f.write_fmt(const_format_args!("{NAME}::LoadFalse {register_addr:#x}"))
            }

            Instruction::Load_u8 { register_addr, val } => f.write_fmt(const_format_args!(
                "{NAME}::Load_u8 {register_addr:#x} {val}({val:#x})"
            )),
            Instruction::Load_u64 { register_addr, val } => f.write_fmt(const_format_args!(
                "{NAME}::Load_u64 {register_addr:#x} {val}({val:#x})"
            )),
            Instruction::LoadThis { register_addr } => {
                f.write_fmt(const_format_args!("{NAME}::LoadThis {register_addr:#x}"))
            }
            Instruction::Load_String { register_addr, val } => f.write_fmt(const_format_args!(
                "{NAME}::Load_String {register_addr:#x} {val}"
            )),

            Instruction::NewObject {
                ty,
                ctor_name,
                args,
                register_addr,
            } => f.write_fmt(const_format_args!(
                "{NAME}::NewObject {ty} {ctor_name} {args} {register_addr:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),

            Instruction::InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => f.write_fmt(const_format_args!(
                "{NAME}::InstanceCall {val:#x} {method} {args} {ret_at:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )),
            Instruction::StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => f.write_fmt(const_format_args!(
                "{NAME}::StaticCall {ty} {method} {args} {ret_at:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )),

            Instruction::LoadArg { register_addr, arg } => f.write_fmt(const_format_args!(
                "{NAME}::LoadArg {register_addr:#x} {arg:#x}"
            )),
            Instruction::LoadStatic {
                register_addr,
                ty,
                field,
            } => f.write_fmt(const_format_args!(
                "{NAME}::LoadStatic {register_addr:#x} {ty} {field}"
            )),

            Instruction::SetThisField { val_addr, field } => f.write_fmt(const_format_args!(
                "{NAME}::SetThisField {val_addr:#x} {field}"
            )),
            Instruction::SetStaticField {
                val_addr,
                ty,
                field,
            } => f.write_fmt(const_format_args!(
                "{NAME}::SetStaticField {val_addr:#x} {ty} {field}"
            )),
            Instruction::Throw { exception_addr } => {
                f.write_fmt(const_format_args!("{NAME}::Throw {exception_addr:#x}"))
            }
            Instruction::ReturnVal { register_addr } => {
                f.write_fmt(const_format_args!("{NAME}::ReturnVal {register_addr:#x}"))
            }
        }
    }
}
