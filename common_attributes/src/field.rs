use binary_proc_macros::{ReadFromSection, WriteToSection};
use derive_ctor::ctor;
use enumflags2::{BitFlags, bitflags};
use getset::{CopyGetters, MutGetters, Setters};

use crate::Visibility;

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum FieldImplementationFlags {
    Static,
}

#[derive(
    Clone,
    Copy,
    Debug,
    ctor,
    CopyGetters,
    Setters,
    MutGetters,
    ReadFromSection,
    WriteToSection,
    serde::Serialize,
    serde::Deserialize,
)]
#[ctor(pub new)]
#[getset(set = "pub", get_mut = "pub")]
#[get_copy = "pub"]
#[serde(deny_unknown_fields)]
pub struct FieldAttr {
    vis: Visibility,
    impl_flags: BitFlags<FieldImplementationFlags>,
}

impl FieldAttr {
    pub fn is_static(&self) -> bool {
        self.impl_flags.contains(FieldImplementationFlags::Static)
    }
}
