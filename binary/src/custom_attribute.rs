use global::{StringName, WithType, getset::Getters};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::item_token::{MethodToken, TypeToken};

#[derive(Debug, Clone, Copy, WithType, ReadFromFile, WriteToFile)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromFile, WriteToFile))]
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

#[derive(Debug, Clone, WithType, ReadFromFile, WriteToFile)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromFile, WriteToFile))]
pub enum CustomAttributeValue {
    Boolean(bool),
    Char(char),
    Integer(Integer),
    // TODO: float, double and System.Object are not supported yet
    String(StringName),
    SystemType(TypeToken),
    PureEnum { ty: TypeToken, val: Integer },
}

#[derive(Debug, Clone, ReadFromFile, WriteToFile, Getters)]
#[getset(get = "pub")]
pub struct CustomAttribute {
    ty: TypeToken,
    ctor_name: MethodToken,
    positional_args: Vec<CustomAttributeValue>,
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
