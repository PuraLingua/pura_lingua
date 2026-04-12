use std::fmt::Debug;

use binary_core::traits::{ReadFromSection, WriteToSection};
use binary_proc_macros::{ReadFromSection, WriteToSection};

mod sealed {
    use crate::instruction::{RegisterAddr, ShortRegisterAddr};

    pub trait IRegisterSealed {}
    impl IRegisterSealed for RegisterAddr {}
    impl IRegisterSealed for ShortRegisterAddr {}
}

pub const trait IRegisterAddr:
    sealed::IRegisterSealed
    + Copy
    + const Clone
    + const Default
    + ReadFromSection
    + WriteToSection
    + Debug
    + std::fmt::Display
    + std::fmt::LowerHex
    + std::fmt::UpperHex
{
    type TInner;
    const ZERO: Self;

    fn new(x: Self::TInner) -> Self;
    fn get(self) -> u64;
    fn into_generic(self) -> RegisterAddr;
    #[inline(always)]
    fn get_usize(self) -> usize {
        self.get() as usize
    }
}

/// Even though it has repr(transparent), the layout is opaque and may change in the future
#[repr(transparent)]
#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
#[derive_const(Clone)]
pub struct RegisterAddr(u64);

impl RegisterAddr {
    pub const fn try_into_short(self) -> Option<ShortRegisterAddr> {
        u16::try_from(self.0).ok().map(ShortRegisterAddr)
    }
}

/// Even though it has repr(transparent), the layout is opaque and may change in the future
#[repr(transparent)]
#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
#[derive_const(Clone)]
pub struct ShortRegisterAddr(u16);

impl const IRegisterAddr for RegisterAddr {
    type TInner = u64;
    const ZERO: Self = Self(0);

    #[inline(always)]
    fn new(x: u64) -> Self {
        Self(x)
    }
    #[inline(always)]
    fn get(self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn into_generic(self) -> RegisterAddr {
        self
    }
}
impl const IRegisterAddr for ShortRegisterAddr {
    type TInner = u16;
    const ZERO: Self = Self(0);

    #[inline(always)]
    fn new(x: Self::TInner) -> Self {
        Self(x)
    }
    #[inline(always)]
    fn get(self) -> u64 {
        self.0 as u64
    }
    #[inline(always)]
    fn into_generic(self) -> RegisterAddr {
        RegisterAddr(self.get())
    }
}

impl const Default for RegisterAddr {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl const Default for ShortRegisterAddr {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

macro fmt_impl($($Trait:ident)*) {$(
	impl std::fmt::$Trait for RegisterAddr {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			std::fmt::$Trait::fmt(&self.get(), f)
		}
	}
	impl std::fmt::$Trait for ShortRegisterAddr {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			std::fmt::$Trait::fmt(&self.get(), f)
		}
	}
)*}

fmt_impl!(
    Display
    LowerHex
    UpperHex
);

macro op_impl($(
	$Trait:ident $f:ident
)*) {$(
	impl std::ops::$Trait for RegisterAddr {
		type Output = Self;
		fn $f(self, rhs: Self) -> Self::Output {
			Self(self.0.$f(rhs.0))
		}
	}
	impl std::ops::$Trait<u64> for RegisterAddr {
		type Output = Self;
		fn $f(self, rhs: u64) -> Self::Output {
			Self(self.0.$f(rhs))
		}
	}

	impl std::ops::$Trait for ShortRegisterAddr {
		type Output = Self;
		fn $f(self, rhs: Self) -> Self::Output {
			Self(self.0.$f(rhs.0))
		}
	}
	impl std::ops::$Trait<u16> for ShortRegisterAddr {
		type Output = Self;
		fn $f(self, rhs: u16) -> Self::Output {
			Self(self.0.$f(rhs))
		}
	}
)*}

macro op_assign_impl($(
	$Trait:ident $f:ident
)*) {$(
	impl std::ops::$Trait for RegisterAddr {
		fn $f(&mut self, rhs: Self) {
			self.0.$f(rhs.0);
		}
	}
	impl std::ops::$Trait<u64> for RegisterAddr {
		fn $f(&mut self, rhs: u64) {
			self.0.$f(rhs);
		}
	}

	impl std::ops::$Trait for ShortRegisterAddr {
		fn $f(&mut self, rhs: Self) {
			self.0.$f(rhs.0);
		}
	}
	impl std::ops::$Trait<u16> for ShortRegisterAddr {
		fn $f(&mut self, rhs: u16) {
			self.0.$f(rhs);
		}
	}
)*}

op_impl!(
    Add add
    Sub sub
);

op_assign_impl!(
    AddAssign add_assign
    SubAssign sub_assign
);
