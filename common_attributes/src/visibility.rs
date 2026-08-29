use binary_proc_macros::{ReadFromSection, WriteToSection};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(
    Clone,
    Copy,
    Debug,
    TryFromPrimitive,
    IntoPrimitive,
    Eq,
    PartialEq,
    ReadFromSection,
    WriteToSection,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum Visibility {
    Public,
    Private,
    AssemblyOnly,
}
