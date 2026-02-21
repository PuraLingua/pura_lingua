use std::ops::{Add, Sub};

pub trait CompressivePrimitive {
    type Compressed;
}

impl CompressivePrimitive for u32 {
    type Compressed = CompressedU32;
}

pub type Compressed<T> = <T as CompressivePrimitive>::Compressed;

#[derive(Copy)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompressedU32(pub u32);

impl CompressedU32 {
    pub const fn is_valid(self) -> bool {
        (Self::MIN..=Self::MAX).contains(&self)
    }
}

macro impl_op($($T:ty => $f:ident)*) {$(
	impl const $T for CompressedU32 {
		type Output = Self;

		fn $f(self, rhs: Self) -> Self::Output {
			Self(self.0.$f(rhs.0))
		}
	}
)*}

impl_op! {
    Add => add
    Sub => sub
}

impl CompressedU32 {
    pub const MAX: Self = Self(0b11111111111111111111111111111);
    pub const MIN: Self = Self(u32::MIN);
}
