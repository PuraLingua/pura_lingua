use std::fmt::Display;

use binary_proc_macros::{ReadFromSection, WriteToSection};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
pub struct CommonReadPointerTo<TRegisterAddr: IRegisterAddr> {
    pub ptr: TRegisterAddr,
    pub size: TRegisterAddr,
    pub destination: TRegisterAddr,
}

impl CommonReadPointerTo<RegisterAddr> {
    pub const fn try_into_short(self) -> Result<CommonReadPointerTo<ShortRegisterAddr>, Self> {
        let Self {
            ptr,
            size,
            destination,
        } = self;
        struct ComposeSizeWithPtr(/* ptr */ ShortRegisterAddr);
        impl const FnOnce<(ShortRegisterAddr,)> for ComposeSizeWithPtr {
            type Output = (ShortRegisterAddr, ShortRegisterAddr);
            #[inline(always)]
            extern "rust-call" fn call_once(self, (size,): (ShortRegisterAddr,)) -> Self::Output {
                (self.0, size)
            }
        }
        struct Construct {
            ptr: ShortRegisterAddr,
            size: ShortRegisterAddr,
        }
        impl const FnOnce<(ShortRegisterAddr,)> for Construct {
            type Output = CommonReadPointerTo<ShortRegisterAddr>;
            #[inline(always)]
            extern "rust-call" fn call_once(
                self,
                (destination,): (ShortRegisterAddr,),
            ) -> Self::Output {
                CommonReadPointerTo {
                    ptr: self.ptr,
                    size: self.size,
                    destination,
                }
            }
        }

        struct ThenSize(/* size */ RegisterAddr);
        impl const FnOnce<(ShortRegisterAddr,)> for ThenSize {
            type Output = Option<(ShortRegisterAddr, ShortRegisterAddr)>;
            #[inline(always)]
            extern "rust-call" fn call_once(self, (ptr,): (ShortRegisterAddr,)) -> Self::Output {
                self.0.try_into_short().map(ComposeSizeWithPtr(ptr))
            }
        }
        struct ThenDestination(/* destination */ RegisterAddr);
        impl const FnOnce<((ShortRegisterAddr, ShortRegisterAddr),)> for ThenDestination {
            type Output = Option<CommonReadPointerTo<ShortRegisterAddr>>;
            #[inline(always)]
            extern "rust-call" fn call_once(
                self,
                ((ptr, size),): ((ShortRegisterAddr, ShortRegisterAddr),),
            ) -> Self::Output {
                self.0.try_into_short().map(Construct { ptr, size })
            }
        }

        ptr.try_into_short()
            .and_then(ThenSize(size))
            .and_then(ThenDestination(destination))
            .ok_or(self)
    }
}

impl<TRegisterAddr: IRegisterAddr> const Clone for CommonReadPointerTo<TRegisterAddr> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<TRegisterAddr: IRegisterAddr> Display for CommonReadPointerTo<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} -> {} of size {}",
            self.ptr, self.destination, self.size
        )
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

impl CommonWritePointer<RegisterAddr> {
    pub const fn try_into_short(self) -> Result<CommonWritePointer<ShortRegisterAddr>, Self> {
        let Self { source, size, ptr } = self;
        struct ComposeSizeWithSource(/* source */ ShortRegisterAddr);
        impl const FnOnce<(ShortRegisterAddr,)> for ComposeSizeWithSource {
            type Output = (ShortRegisterAddr, ShortRegisterAddr);
            #[inline(always)]
            extern "rust-call" fn call_once(self, (size,): (ShortRegisterAddr,)) -> Self::Output {
                (self.0, size)
            }
        }
        struct Construct {
            source: ShortRegisterAddr,
            size: ShortRegisterAddr,
        }
        impl const FnOnce<(ShortRegisterAddr,)> for Construct {
            type Output = CommonWritePointer<ShortRegisterAddr>;
            #[inline(always)]
            extern "rust-call" fn call_once(self, (ptr,): (ShortRegisterAddr,)) -> Self::Output {
                CommonWritePointer {
                    source: self.source,
                    size: self.size,
                    ptr,
                }
            }
        }

        struct ThenSize(/* size */ RegisterAddr);
        impl const FnOnce<(ShortRegisterAddr,)> for ThenSize {
            type Output = Option<(ShortRegisterAddr, ShortRegisterAddr)>;
            #[inline(always)]
            extern "rust-call" fn call_once(self, (source,): (ShortRegisterAddr,)) -> Self::Output {
                self.0.try_into_short().map(ComposeSizeWithSource(source))
            }
        }
        struct ThenPtr(/* ptr */ RegisterAddr);
        impl const FnOnce<((ShortRegisterAddr, ShortRegisterAddr),)> for ThenPtr {
            type Output = Option<CommonWritePointer<ShortRegisterAddr>>;
            #[inline(always)]
            extern "rust-call" fn call_once(
                self,
                ((source, size),): ((ShortRegisterAddr, ShortRegisterAddr),),
            ) -> Self::Output {
                self.0.try_into_short().map(Construct { source, size })
            }
        }

        source
            .try_into_short()
            .and_then(ThenSize(size))
            .and_then(ThenPtr(ptr))
            .ok_or(self)
    }
}

impl<TRegisterAddr: IRegisterAddr> const Clone for CommonWritePointer<TRegisterAddr> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<TRegisterAddr: IRegisterAddr> Display for CommonWritePointer<TRegisterAddr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} of size {} -> {}", self.source, self.size, self.ptr)
    }
}

pub type WritePointer = CommonWritePointer<RegisterAddr>;
pub type SWritePointer = CommonWritePointer<ShortRegisterAddr>;
