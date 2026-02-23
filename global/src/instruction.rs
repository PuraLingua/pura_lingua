#![allow(non_camel_case_types)]

use std::fmt::Display;
use std::marker::Destruct;
use std::ptr::NonNull;

use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use proc_macros::WithType;

mod jumping;
pub use jumping::*;

use crate::non_purus_call_configuration::NonPurusCallConfiguration;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, ReadFromSection, WriteToSection)]
pub struct RegisterAddr(u64);

impl RegisterAddr {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[repr(u64)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction<TString, TTypeRef, TMethodRef, TFieldRef> {
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
    Load_u16 {
        register_addr: u64,
        val: u16,
    },
    Load_u32 {
        register_addr: u64,
        val: u32,
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
        val: TString,
    },

    LoadTypeValueSize {
        register_addr: u64,
        ty: TTypeRef,
    },

    ReadPointerTo {
        ptr: u64,
        size: u64,
        destination: u64,
    },
    WritePointer {
        source: u64,
        size: u64,
        ptr: u64,
    },

    IsNull {
        register_addr: u64,
        to_check: u64,
    },

    NewObject {
        ty: TTypeRef,
        ctor_name: TMethodRef,
        args: Vec<u64>,
        register_addr: u64,
    },
    NewArray {
        element_type: TTypeRef,
        len: u64,
        register_addr: u64,
    },
    NewDynamicArray {
        element_type: TTypeRef,
        len_addr: u64,
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
    StaticNonPurusCall {
        f_pointer: u64,
        config: NonPurusCallConfiguration,
        args: Vec<u64>,
        ret_at: u64,
    },
    DynamicNonPurusCall {
        f_pointer: u64,
        config: u64,
        args: Vec<u64>,
        ret_at: u64,
    },

    LoadNonPurusCallConfiguration {
        register_addr: u64,
        val: NonPurusCallConfiguration,
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

    LoadField {
        container: u64,
        field: TFieldRef,
        register_addr: u64,
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

    Jump {
        target: JumpTarget,
    },

    JumpIf {
        register_addr: u64,
        target: JumpTarget,
    },

    JumpIfNull {
        to_check: u64,
        target: JumpTarget,
    },
    JumpIfNotNull {
        to_check: u64,
        target: JumpTarget,
    },
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
                .byte_add(size_of::<u64>())
        }
    }
}

pub macro instruction_match_helper(
    $this:expr,
    $str_help:ident,
    $type_help:ident,
    $method_help:ident,
    $field_help:ident,
    $success:ident $(,)?
) {{
    use $crate::instruction::Instruction::*;
    match $this {
        LoadTrue { register_addr } => $success(LoadTrue { register_addr }),
        LoadFalse { register_addr } => $success(LoadFalse { register_addr }),
        Load_u8 { register_addr, val } => $success(Load_u8 { register_addr, val }),
        Load_u16 { register_addr, val } => $success(Load_u16 { register_addr, val }),
        Load_u32 { register_addr, val } => $success(Load_u32 { register_addr, val }),
        Load_u64 { register_addr, val } => $success(Load_u64 { register_addr, val }),
        LoadThis { register_addr } => $success(LoadThis { register_addr }),
        Load_String { register_addr, val } => $success(Load_String {
            register_addr,
            val: $str_help!(val),
        }),
        LoadTypeValueSize { register_addr, ty } => $success(LoadTypeValueSize {
            register_addr,
            ty: $type_help!(ty),
        }),
        ReadPointerTo {
            ptr,
            size,
            destination,
        } => $success(ReadPointerTo {
            ptr,
            size,
            destination,
        }),
        WritePointer { source, size, ptr } => $success(WritePointer { source, size, ptr }),

        IsNull {
            register_addr,
            to_check,
        } => $success(IsNull {
            register_addr,
            to_check,
        }),

        NewObject {
            ty,
            ctor_name,
            args,
            register_addr,
        } => $success(NewObject {
            ty: $type_help!(ty),
            ctor_name: $method_help!(ctor_name),
            args,
            register_addr,
        }),
        NewArray {
            element_type,
            len,
            register_addr,
        } => $success(NewArray {
            element_type: $type_help!(element_type),
            len,
            register_addr,
        }),
        NewDynamicArray {
            element_type,
            len_addr,
            register_addr,
        } => $success(NewDynamicArray {
            element_type: $type_help!(element_type),
            len_addr,
            register_addr,
        }),

        InstanceCall {
            val,
            method,
            args,
            ret_at,
        } => $success(InstanceCall {
            val,
            method: $method_help!(method),
            args,
            ret_at,
        }),
        StaticCall {
            ty,
            method,
            args,
            ret_at,
        } => $success(StaticCall {
            ty: $type_help!(ty),
            method: $method_help!(method),
            args,
            ret_at,
        }),
        StaticNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        } => $success(StaticNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        }),
        DynamicNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        } => $success(DynamicNonPurusCall {
            f_pointer,
            config,
            args,
            ret_at,
        }),

        LoadNonPurusCallConfiguration { register_addr, val } => {
            $success(LoadNonPurusCallConfiguration { register_addr, val })
        }

        LoadArg { register_addr, arg } => $success(LoadArg { register_addr, arg }),
        LoadStatic {
            register_addr,
            ty,
            field,
        } => $success(LoadStatic {
            register_addr,
            ty: $type_help!(ty),
            field: $field_help!(field),
        }),
        LoadField {
            container,
            field,
            register_addr,
        } => $success(LoadField {
            container,
            field: $field_help!(field),
            register_addr,
        }),
        SetThisField { val_addr, field } => $success(SetThisField {
            val_addr,
            field: $field_help!(field),
        }),
        SetStaticField {
            val_addr,
            ty,
            field,
        } => $success(SetStaticField {
            val_addr,
            ty: $type_help!(ty),
            field: $field_help!(field),
        }),
        Throw { exception_addr } => $success(Throw { exception_addr }),
        ReturnVal { register_addr } => $success(ReturnVal { register_addr }),

        Jump { target } => $success(Jump { target }),

        JumpIf {
            register_addr,
            target,
        } => $success(JumpIf {
            register_addr,
            target,
        }),

        JumpIfNull { to_check, target } => $success(JumpIfNull { to_check, target }),
        JumpIfNotNull { to_check, target } => $success(JumpIfNotNull { to_check, target }),
    }
}}

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<Option<TString>, Option<TTypeRef>, Option<TMethodRef>, Option<TFieldRef>>
{
    pub const fn transpose(self) -> Option<Instruction<TString, TTypeRef, TMethodRef, TFieldRef>>
    where
        Self: [const] Destruct,
    {
        macro m_help($val:ident) {
            match $val {
                Some($val) => $val,
                None => return None,
            }
        }
        instruction_match_helper!(self, m_help, m_help, m_help, m_help, Some)
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
    pub const fn transpose<UniE>(
        self,
    ) -> Result<Instruction<TString, TTypeRef, TMethodRef, TFieldRef>, UniE>
    where
        Self: [const] Destruct,
        UniE: [const] From<E1> + [const] From<E2> + [const] From<E3> + [const] From<E4>,
    {
        macro m_help($val:ident) {
            match $val {
                Ok($val) => $val,
                Err(e) => return Err(e.into()),
            }
        }
        instruction_match_helper!(self, m_help, m_help, m_help, m_help, Ok)
    }
}

#[inline(always)]
const fn noop<T>(v: T) -> T {
    v
}

impl<TString, TTypeRef, TMethodRef, TFieldRef>
    Instruction<TString, TTypeRef, TMethodRef, TFieldRef>
{
    pub const fn map_types<
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
        mut f_string: FString,
        mut f_type: FTypeRef,
        mut f_method: FMethodRef,
        mut f_field: FFieldRef,
    ) -> Instruction<_TString, _TTypeRef, _TMethodRef, _TFieldRef>
    where
        FString: [const] FnMut(TString) -> _TString + [const] Destruct,
        FTypeRef: [const] FnMut(TTypeRef) -> _TTypeRef + [const] Destruct,
        FMethodRef: [const] FnMut(TMethodRef) -> _TMethodRef + [const] Destruct,
        FFieldRef: [const] FnMut(TFieldRef) -> _TFieldRef + [const] Destruct,
        Self: [const] Destruct,
    {
        macro str_help($val:ident) {
            (f_string)($val)
        }
        macro type_help($val:ident) {
            (f_type)($val)
        }
        macro method_help($val:ident) {
            (f_method)($val)
        }
        macro field_help($val:ident) {
            (f_field)($val)
        }
        instruction_match_helper!(
            self,
            str_help,
            type_help,
            method_help,
            field_help,
            /* success */ noop
        )
    }
    pub const fn map_string<_TString, FString>(
        self,
        f_string: FString,
    ) -> Instruction<_TString, TTypeRef, TMethodRef, TFieldRef>
    where
        FString: [const] FnMut(TString) -> _TString + [const] Destruct,
        Self: [const] Destruct,
    {
        self.map_types(f_string, noop, noop, noop)
    }
    pub const fn map_type_ref<_TTypeRef, FTypeRef>(
        self,
        f_type: FTypeRef,
    ) -> Instruction<TString, _TTypeRef, TMethodRef, TFieldRef>
    where
        FTypeRef: [const] FnMut(TTypeRef) -> _TTypeRef + [const] Destruct,
        Self: [const] Destruct,
    {
        self.map_types(noop, f_type, noop, noop)
    }
    pub const fn map_method_ref<_TMethodRef, FMethodRef>(
        self,
        f_method: FMethodRef,
    ) -> Instruction<TString, TTypeRef, _TMethodRef, TFieldRef>
    where
        FMethodRef: [const] FnMut(TMethodRef) -> _TMethodRef + [const] Destruct,
        Self: [const] Destruct,
    {
        self.map_types(noop, noop, f_method, noop)
    }
    pub const fn map_field_ref<_TFieldRef, FFieldRef>(
        self,
        f_field: FFieldRef,
    ) -> Instruction<TString, TTypeRef, TMethodRef, _TFieldRef>
    where
        FFieldRef: [const] FnMut(TFieldRef) -> _TFieldRef + [const] Destruct,
        Self: [const] Destruct,
    {
        self.map_types(noop, noop, noop, f_field)
    }
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
        const NAME: &str = "Instruction";
        use Instruction::*;
        match self {
            LoadTrue { register_addr } => write!(f, "{NAME}::LoadTrue {register_addr:#x}"),
            LoadFalse { register_addr } => write!(f, "{NAME}::LoadFalse {register_addr:#x}"),

            Load_u8 { register_addr, val } => {
                write!(f, "{NAME}::Load_u8 {register_addr:#x} {val}({val:#x})")
            }
            Load_u16 { register_addr, val } => {
                write!(f, "{NAME}::Load_u16 {register_addr:#x} {val}({val:#x})")
            }
            Load_u32 { register_addr, val } => {
                write!(f, "{NAME}::Load_u32 {register_addr:#x} {val}({val:#x})")
            }
            Load_u64 { register_addr, val } => {
                write!(f, "{NAME}::Load_u64 {register_addr:#x} {val}({val:#x})")
            }
            LoadThis { register_addr } => write!(f, "{NAME}::LoadThis {register_addr:#x}"),
            Load_String { register_addr, val } => {
                write!(f, "{NAME}::Load_String {register_addr:#x} {val}")
            }
            LoadTypeValueSize { register_addr, ty } => {
                write!(f, "{NAME}::LoadTypeValueSize {register_addr:#x} {ty}")
            }
            ReadPointerTo {
                ptr,
                size,
                destination,
            } => {
                write!(
                    f,
                    "{NAME}::ReadPointerTo {ptr:#x} {size:#x} {destination:#x}"
                )
            }
            WritePointer { source, size, ptr } => {
                write!(f, "{NAME}::WritePointer {source:#x} {ptr:#x} {size:#x}")
            }

            IsNull {
                register_addr,
                to_check,
            } => write!(f, "{NAME}::IsNull {register_addr:#x} {to_check:#x}"),

            NewObject {
                ty,
                ctor_name,
                args,
                register_addr,
            } => write!(
                f,
                "{NAME}::NewObject {ty} {ctor_name} {args} {register_addr:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            NewArray {
                element_type,
                len,
                register_addr,
            } => write!(
                f,
                "{NAME}::NewArray {element_type} {len} {register_addr:#x}",
            ),
            NewDynamicArray {
                element_type,
                len_addr,
                register_addr,
            } => write!(
                f,
                "{NAME}::NewArray {element_type} {len_addr:#x} {register_addr:#x}",
            ),

            InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => write!(
                f,
                "{NAME}::InstanceCall {val:#x} {method} {args} {ret_at:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            StaticCall {
                ty,
                method,
                args,
                ret_at,
            } => write!(
                f,
                "{NAME}::StaticCall {ty} {method} {args} {ret_at:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            StaticNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => write!(
                f,
                "{NAME}::StaticNonPurusCall {ret_at:#x} {f_pointer:#x}({args}) {config:#?}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            DynamicNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => write!(
                f,
                "{NAME}::DynamicNonPurusCall {ret_at:#x} {f_pointer:#x}({args}) {config:#x}",
                args = args
                    .iter()
                    .map(|x| format!("{x:#x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            LoadNonPurusCallConfiguration { register_addr, val } => {
                write!(
                    f,
                    "{NAME}::LoadNonPurusCallConfiguration {register_addr:#x} {val:?}"
                )
            }

            LoadArg { register_addr, arg } => {
                write!(f, "{NAME}::LoadArg {register_addr:#x} {arg:#x}")
            }
            LoadStatic {
                register_addr,
                ty,
                field,
            } => write!(f, "{NAME}::LoadStatic {register_addr:#x} {ty} {field}"),

            LoadField {
                container,
                field,
                register_addr,
            } => write!(
                f,
                "{NAME}::LoadField {container:#x} {field} {register_addr:#x}"
            ),

            SetThisField { val_addr, field } => {
                write!(f, "{NAME}::SetThisField {val_addr:#x} {field}")
            }
            SetStaticField {
                val_addr,
                ty,
                field,
            } => write!(f, "{NAME}::SetStaticField {val_addr:#x} {ty} {field}"),
            Throw { exception_addr } => write!(f, "{NAME}::Throw {exception_addr:#x}"),
            ReturnVal { register_addr } => write!(f, "{NAME}::ReturnVal {register_addr:#x}"),

            Jump { target } => write!(f, "{NAME}::Jump {target}"),

            JumpIf {
                register_addr,
                target,
            } => f.write_fmt(const_format_args!(
                "{NAME}::JumpIf {register_addr:#x} {target}"
            )),

            JumpIfNull { to_check, target } => {
                write!(f, "{NAME}::JumpIfNull {to_check:#x} {target}")
            }
            JumpIfNotNull { to_check, target } => {
                write!(f, "{NAME}::JumpIfNotNull {to_check:#x} {target}")
            }
        }
    }
}
