//! Like .NET's TypeToken
//!

use std::fmt::{Debug, Display};

use bitfields::{FromBits, IntoBits, bitfield};
use derive_more::Display;
use global_proc_macros::AllVariants;

macro impl_for_tokens($($i:ident)*) {$(
    impl PartialEq for $i {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for $i {}

    impl Display for $i {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({}){}", self.ty(), self.index())
        }
    }

    impl ::std::hash::Hash for $i {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.hash(state)
        }
    }
)*}

macro impl_for_sub_item_tokens($($i:ident)*) {$(
	impl $i {
		pub const fn into_item(self) -> ItemToken {
			unsafe { std::mem::transmute(self) }
		}
		pub const fn as_item(&self) -> &ItemToken {
			unsafe { (self as *const Self as *const ItemToken).as_ref_unchecked() }
		}
	}
)*}

macro impl_for_tokens_types($($i:ident)*) {$(
    ty_from_into_bits!($i);
)*}

#[derive(Clone, Copy)]
#[bitfield(u32, from_endian = little, into_endian = little, builder = true)]
pub struct ItemToken {
    #[bits(8)]
    pub ty: ItemType,
    #[bits(24)]
    pub index: u32,
}

#[derive(Clone, Copy)]
#[bitfield(u32, from_endian = little, into_endian = little, builder = true)]
pub struct TypeToken {
    #[bits(8)]
    pub ty: TypeType,
    #[bits(24)]
    pub index: u32,
}

#[derive(Clone, Copy)]
#[bitfield(u32, from_endian = little, into_endian = little, builder = true)]
pub struct MethodToken {
    #[bits(8, default = MethodType::Method)]
    pub ty: MethodType,
    #[bits(24)]
    pub index: u32,
}

impl_for_sub_item_tokens! {
    TypeToken
    MethodToken
}

impl_for_tokens! {
    ItemToken
    TypeToken
    MethodToken
}

#[repr(u8)]
#[derive(Clone, Copy, AllVariants, Debug, Display)]
pub enum MethodType {
    Method = 0x03,
    MethodSpec = 0x04,
    MethodByRuntime = 0x05,
}

#[repr(u8)]
#[derive(Clone, Copy, AllVariants, Debug, Display)]
pub enum TypeType {
    TypeDef = 0x00,
    TypeRef = 0x01,
    TypeSpec = 0x02,

    Generic = 0xFF,
}

#[repr(u8)]
#[derive(Clone, Copy, AllVariants, Debug, Display)]
pub enum ItemType {
    TypeDef = 0x00,
    TypeRef = 0x01,
    TypeSpec = 0x02,

    Method = 0x03,
    MethodSpec = 0x04,
    MethodByRuntime = 0x05,

    Field = 0x06,

    Generic = 0xFF,
}

impl_for_tokens_types! {
    TypeType
    MethodType
    ItemType
}

impl ItemType {
    pub const fn is_type(self) -> bool {
        let mut ind = 0;
        while ind < TypeType::ALL_VARIANTS.len() {
            if TypeType::ALL_VARIANTS[ind] as u8 == self as u8 {
                return true;
            }
            ind += 1;
        }

        false
    }
}

macro ty_from_into_bits($t:ty) {
    impl const ::bitfields::FromBits for $t {
        type Number = u8;
        fn from_bits(i: u8) -> Self {
            check_enum_cast!((Self)i);

            unsafe { std::mem::transmute(i) }
        }
    }
    impl const ::bitfields::IntoBits for $t {
        type Number = u8;
        fn into_bits(self) -> u8 {
            self as u8
        }
    }
}

macro check_enum_cast(($t:ty) $i:expr) {
    #[cfg(debug_assertions)] // Disable checking in release to speed up.
    {
        let mut ind = 0;
        let mut found = false;
        while ind < <$t>::ALL_VARIANTS.len() {
            found = $i == <$t>::ALL_VARIANTS[ind] as _;
            if found {
                break;
            }
            ind += 1;
        }
        if !found {
            panic!("Cannot cast to ItemType");
        }
    }
}
