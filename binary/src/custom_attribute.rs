use binary_core::traits::StringRef;
use global::WithType;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::item_token::{MethodToken, TypeToken};

#[derive(Debug, Clone, Copy, WithType, ReadFromSection, WriteToSection)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromSection, WriteToSection))]
pub enum Integer {
    Byte(u8),
    SByte(i8),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
}

#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromSection, WriteToSection))]
pub enum CustomAttributeValue {
    Boolean(bool),
    Char(char),
    Integer(Integer),
    // TODO: float, double and System.Object are not supported yet
    String(StringRef),
    SystemType(TypeToken),
    PureEnum { ty: TypeToken, val: Integer },
}

#[derive(Debug, Clone, ReadFromSection, WriteToSection)]
pub struct CustomAttribute {
    pub ty: TypeToken,
    pub ctor_name: MethodToken,
    pub positional_args: Vec<CustomAttributeValue>,
}

impl CustomAttribute {
    pub fn new(
        ty: TypeToken,
        ctor_name: MethodToken,
        positional_args: Vec<CustomAttributeValue>,
    ) -> Self {
        Self {
            ty,
            ctor_name,
            positional_args,
        }
    }
}
