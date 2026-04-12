use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use proc_macros::{DeriveMap, Transpose, WithType};

use crate::{instruction::IRegisterAddr, non_purus_call_configuration::NonPurusCallConfiguration};

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> {
    pub addr: TRegisterAddr,
    pub content: LoadContent<TString, TTypeRef, TFieldRef, TRegisterAddr>,
}

impl<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr>
    Instruction_Load<Option<TString>, Option<TTypeRef>, Option<TFieldRef>, TRegisterAddr>
{
    pub fn transpose(
        self,
    ) -> Option<Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>> {
        let Self { addr, content } = self;
        match content.transpose() {
            Some(content) => Some(Instruction_Load { addr, content }),
            None => None,
        }
    }
}

impl<TString, E1, TTypeRef, E2, TFieldRef, E3, TRegisterAddr: IRegisterAddr>
    Instruction_Load<
        Result<TString, E1>,
        Result<TTypeRef, E2>,
        Result<TFieldRef, E3>,
        TRegisterAddr,
    >
{
    pub fn transpose<UniE>(
        self,
    ) -> Result<Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>, UniE>
    where
        UniE: From<E1> + From<E2> + From<E3>,
    {
        let Self { addr, content } = self;
        content
            .transpose()
            .map(|content| Instruction_Load { addr, content })
    }
}

impl<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr>
    Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
{
    #[allow(nonstandard_style)]
    pub fn map<
        __TString,
        __TTypeRef,
        __TFieldRef,
        __TRegisterAddr,
        __F_TString,
        __F_TTypeRef,
        __F_TFieldRef,
        __F_TRegisterAddr,
    >(
        self,
        f_TString: __F_TString,
        f_TTypeRef: __F_TTypeRef,
        f_TFieldRef: __F_TFieldRef,
        mut f_TRegisterAddr: __F_TRegisterAddr,
    ) -> Instruction_Load<__TString, __TTypeRef, __TFieldRef, __TRegisterAddr>
    where
        __TRegisterAddr: IRegisterAddr,
        __F_TString: ::core::ops::FnMut(TString) -> __TString,
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        __F_TFieldRef: ::core::ops::FnMut(TFieldRef) -> __TFieldRef,
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
    {
        let Self { addr, content } = self;
        Instruction_Load {
            addr: f_TRegisterAddr(addr),
            content: content.map(f_TString, f_TTypeRef, f_TFieldRef, f_TRegisterAddr),
        }
    }
}

#[inline(always)]
const fn noop<T>(val: T) -> T {
    val
}

impl<TString, TTypeRef, TFieldRef, TRegisterAddr>
    Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_string<__TString, __F_TString>(
        self,
        f: __F_TString,
    ) -> Instruction_Load<__TString, TTypeRef, TFieldRef, TRegisterAddr>
    where
        __F_TString: ::core::ops::FnMut(TString) -> __TString,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(f, noop, noop, noop)
    }
}
impl<TString, TTypeRef, TFieldRef, TRegisterAddr>
    Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_type_ref<__TTypeRef, __F_TTypeRef>(
        self,
        f: __F_TTypeRef,
    ) -> Instruction_Load<TString, __TTypeRef, TFieldRef, TRegisterAddr>
    where
        __F_TTypeRef: ::core::ops::FnMut(TTypeRef) -> __TTypeRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(noop, f, noop, noop)
    }
}
impl<TString, TTypeRef, TFieldRef, TRegisterAddr>
    Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_field_ref<__TFieldRef, __F_TFieldRef>(
        self,
        f: __F_TFieldRef,
    ) -> Instruction_Load<TString, TTypeRef, __TFieldRef, TRegisterAddr>
    where
        __F_TFieldRef: ::core::ops::FnMut(TFieldRef) -> __TFieldRef,
        TRegisterAddr: IRegisterAddr,
    {
        self.map(noop, noop, f, noop)
    }
}
impl<TString, TTypeRef, TFieldRef, TRegisterAddr>
    Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    pub fn map_register_addr<__TRegisterAddr, __F_TRegisterAddr>(
        self,
        f: __F_TRegisterAddr,
    ) -> Instruction_Load<TString, TTypeRef, TFieldRef, __TRegisterAddr>
    where
        __F_TRegisterAddr: ::core::ops::FnMut(TRegisterAddr) -> __TRegisterAddr,
        __TRegisterAddr: IRegisterAddr,
    {
        self.map(noop, noop, noop, f)
    }
}

impl<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> Display
    for Instruction_Load<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TString: Display,
    TTypeRef: Display,
    TFieldRef: Display,
    TRegisterAddr: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(" {} -> {:#x}", self.content, self.addr))
    }
}

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection, Transpose, DeriveMap)]
#[transpose(TString, TTypeRef, TFieldRef)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum LoadContent<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
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
    /// It will read the value if the arg is passed by ref
    ArgValue(u64),

    Static {
        ty: TTypeRef,
        field: TFieldRef,
    },
    Field {
        container: TRegisterAddr,
        field: TFieldRef,
    },

    CaughtException,
}

impl<TString, TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> Display
    for LoadContent<TString, TTypeRef, TFieldRef, TRegisterAddr>
where
    TString: Display,
    TTypeRef: Display,
    TFieldRef: Display,
    TRegisterAddr: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadContent::True => f.write_str("True"),
            LoadContent::False => f.write_str("False"),

            LoadContent::U8(x) => f.write_fmt(format_args!("{x}u8({x:#x})")),
            LoadContent::U16(x) => f.write_fmt(format_args!("{x}u16({x:#x})")),
            LoadContent::U32(x) => f.write_fmt(format_args!("{x}u32({x:#x})")),
            LoadContent::U64(x) => f.write_fmt(format_args!("{x}u64({x:#x})")),

            LoadContent::I8(x) => f.write_fmt(format_args!("{x}i8({x:#x})")),
            LoadContent::I16(x) => f.write_fmt(format_args!("{x}i16({x:#x})")),
            LoadContent::I32(x) => f.write_fmt(format_args!("{x}i32({x:#x})")),
            LoadContent::I64(x) => f.write_fmt(format_args!("{x}i64({x:#x})")),

            LoadContent::This => f.write_str("This"),
            LoadContent::String(x) => f.write_fmt(format_args!("`{x}`str")),
            LoadContent::TypeValueSize(ty) => f.write_fmt(format_args!("sizeof({ty})")),
            LoadContent::NonPurusCallConfiguration(conf) => {
                f.write_fmt(format_args!("non_purus_call_configuration({conf:?})"))
            }
            LoadContent::Arg(arg) => f.write_fmt(format_args!("arg({arg}({arg:#x}))")),
            LoadContent::ArgValue(arg) => f.write_fmt(format_args!("*arg({arg}({arg:#x}))")),
            LoadContent::Static { ty, field } => {
                f.write_fmt(format_args!("static({field} at {ty})"))
            }
            LoadContent::Field { container, field } => {
                f.write_fmt(format_args!("field({field} at {container})"))
            }
            LoadContent::CaughtException => f.write_fmt(format_args!("caught exception")),
        }
    }
}
