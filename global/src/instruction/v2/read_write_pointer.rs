use binary_proc_macros::{ReadFromSection, WriteToSection};

use crate::instruction::v2::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
pub struct CommonReadPointerTo<TRegisterAddr: IRegisterAddr> {
    pub ptr: TRegisterAddr,
    pub size: TRegisterAddr,
    pub destination: TRegisterAddr,
}

impl<TRegisterAddr: IRegisterAddr> const Clone for CommonReadPointerTo<TRegisterAddr> {
    fn clone(&self) -> Self {
        *self
    }
}

pub type ReadPointerTo = CommonReadPointerTo<RegisterAddr>;
pub type SReadPointerTo = CommonReadPointerTo<ShortRegisterAddr>;

#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
pub struct CommonWritePointer<TRegisterAddr: IRegisterAddr> {
    pub source: TRegisterAddr,
    pub size: TRegisterAddr,
    pub ptr: TRegisterAddr,
}

impl<TRegisterAddr: IRegisterAddr> const Clone for CommonWritePointer<TRegisterAddr> {
    fn clone(&self) -> Self {
        *self
    }
}

pub type WritePointer = CommonWritePointer<RegisterAddr>;
pub type SWritePointer = CommonWritePointer<ShortRegisterAddr>;
