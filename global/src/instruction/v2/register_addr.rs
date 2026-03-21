use std::fmt::Debug;

use binary_core::traits::{ReadFromSection, WriteToSection};
use binary_proc_macros::{ReadFromSection, WriteToSection};

pub trait IRegisterAddr:
    Copy
    + const Clone
    + const Default
    + ReadFromSection
    + WriteToSection
    + Debug
    + std::fmt::Display
    + std::fmt::LowerHex
    + std::fmt::UpperHex
{
}

/// Even though it has repr(transparent), the layout is opaque and may change in the future
#[repr(transparent)]
#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
#[derive_const(Clone)]
pub struct RegisterAddr(u64);

/// Even though it has repr(transparent), the layout is opaque and may change in the future
#[repr(transparent)]
#[derive(Debug, Copy, ReadFromSection, WriteToSection)]
#[derive_const(Clone)]
pub struct ShortRegisterAddr(u16);

impl IRegisterAddr for RegisterAddr {}
impl IRegisterAddr for ShortRegisterAddr {}

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

impl RegisterAddr {
    #[inline(always)]
    pub const fn new(x: u64) -> Self {
        Self(x)
    }
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }
    #[inline(always)]
    pub const fn get_usize(self) -> usize {
        self.0 as usize
    }

    pub const ZERO: Self = Self(0);
}

impl ShortRegisterAddr {
    #[inline(always)]
    pub const fn new(x: u16) -> Self {
        Self(x)
    }
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0 as u64
    }
    #[inline(always)]
    pub const fn get_usize(self) -> usize {
        self.0 as usize
    }

    pub const ZERO: Self = Self(0);
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
