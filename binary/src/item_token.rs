//! Like .NET's TypeToken
//!

use std::fmt::{Debug, Display};

use bitfields::{FromBits, IntoBits, bitfield};
use derive_more::Display;
use global::AllVariants;

pub type TokenInner = u32;

macro impl_for_tokens($($i:ident)*) {$(
    const _: () = {};
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

    impl $crate::binary_core::traits::ReadFromSection for $i {
        fn read_from_section(
			cursor: &mut std::io::Cursor<&$crate::binary_core::section::Section>,
		) -> Result<Self, $crate::binary_core::error::Error> {
            #[inline(always)]
            const fn map(src: TokenInner) -> $i {
                unsafe { std::mem::transmute(src) }
            }
			<TokenInner as $crate::binary_core::traits::ReadFromSection>::read_from_section(cursor).map(map)
		}
    }

    impl $crate::binary_core::traits::WriteToSection for $i {
        fn write_to_section(
            &self,
			cursor: &mut std::io::Cursor<&mut Vec<u8>>,
		) -> Result<(), $crate::binary_core::error::Error> {
			<TokenInner as $crate::binary_core::traits::WriteToSection>::write_to_section(
                unsafe { &*(&raw const *self as *const TokenInner) },
                cursor
            )
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

/// ```rust,ignore
/// enum Parent {
///     A,
///     B,
///     C,
/// }
/// sub_enum_of! {
///     pub enum AAndC of Parent {
///         A,
///         C,
///     }
/// }
/// ```
macro sub_enum_of(
    $(#[$meta:meta])*
    $vis:vis enum $Name:ident of $Parent:ty {
        $(
            $(#[$v_meta:meta])*
            $Variant:ident
        ),* $(,)?
    } $checker:ident
) {
    $(#[$meta])*
    $vis enum $Name {
        $(
            $(#[$v_meta])*
            $Variant = <$Parent>::$Variant as _,
        )*
    }

    impl $Parent {
        $vis const fn $checker(&self) -> bool {
            match self {
                $(
                    Self::$Variant => true,
                )*
                _ => false,
            }
        }
    }
}

sub_enum_of! {
    #[repr(u8)]
    #[derive(Clone, Copy, AllVariants, Debug, Display)]
    pub enum MethodType of ItemType {
        Method,
        MethodSpec,
        MethodByRuntime,
    } is_method
}

sub_enum_of! {
    #[repr(u8)]
    #[derive(Clone, Copy, AllVariants, Debug, Display)]
    pub enum TypeType of ItemType {
        TypeDef,
        TypeRef,
        TypeSpec,

        Generic,
    } is_type
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
