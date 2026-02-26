#![allow(nonstandard_style)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(smart_pointer_try_map)]
#![feature(decl_macro)]

use stdlib_header::CoreTypeId;

use crate::definitions::CoreTypeInfo;

pub mod definitions;

mod core_type_id_serde {
    use std::marker::PhantomData;

    use stdlib_header::CoreTypeId;

    pub fn serialize<S: serde::Serializer>(
        this: &CoreTypeId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serde::Serializer::serialize_unit_variant(
            serializer,
            "CoreTypeId",
            *this as u32,
            this.raw_name(),
        )
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<CoreTypeId, D::Error> {
        #[repr(transparent)]
        struct CoreTypeIdLocal(CoreTypeId);
        #[doc(hidden)]
        struct CoreTypeIdVisitor;

        #[automatically_derived]
        impl<'de> serde::de::Visitor<'de> for CoreTypeIdVisitor {
            type Value = CoreTypeId;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Formatter::write_str(formatter, "variant identifier")
            }
            fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if val >= usize::MAX as u64 {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(val),
                        &"variant index at least less than usize::MAX",
                    ));
                }
                CoreTypeId::try_from(val as u32).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(val),
                        &format!(
                            "variant index should be less than {}",
                            *CoreTypeId::ALL_VARIANTS.last().unwrap() as u32
                        )
                        .as_str(),
                    )
                })
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                for (ind, i) in CoreTypeId::ALL_VARIANTS_NAME.iter().copied().enumerate() {
                    if i == value {
                        return Ok(CoreTypeId::ALL_VARIANTS[ind]);
                    }
                }
                Err(serde::de::Error::unknown_variant(
                    value,
                    &CoreTypeId::ALL_VARIANTS_NAME,
                ))
            }
            fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                for (ind, i) in CoreTypeId::ALL_VARIANTS_NAME.iter().copied().enumerate() {
                    if i.as_bytes() == value {
                        return Ok(CoreTypeId::ALL_VARIANTS[ind]);
                    }
                }
                let value = String::from_utf8_lossy(value);
                Err(serde::de::Error::unknown_variant(
                    &value,
                    &CoreTypeId::ALL_VARIANTS_NAME,
                ))
            }
        }
        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for CoreTypeIdLocal {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserializer::deserialize_identifier(deserializer, CoreTypeIdVisitor)
                    .map(CoreTypeIdLocal)
            }
        }
        #[doc(hidden)]
        struct Visitor<'de> {
            marker: PhantomData<CoreTypeId>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> Visitor<'de> {
            #[inline(always)]
            const fn new() -> Self {
                Self {
                    marker: PhantomData,
                    lifetime: PhantomData,
                }
            }
        }
        #[automatically_derived]
        impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = CoreTypeId;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("enum CoreTypeId")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                let (kind, variants) = serde::de::EnumAccess::variant::<CoreTypeIdLocal>(data)?;
                serde::de::VariantAccess::unit_variant(variants)?;
                Ok(kind.0)
            }
        }
        serde::Deserializer::deserialize_enum(
            deserializer,
            "CoreTypeId",
            &CoreTypeId::ALL_VARIANTS_NAME,
            Visitor::new(),
        )
    }
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CoreTypeRef {
    #[serde(with = "core_type_id_serde")]
    Core(CoreTypeId),
    WithGeneric(#[serde(with = "core_type_id_serde")] CoreTypeId, Vec<Self>),
    Generic(u32),
}

impl From<stdlib_header::CoreTypeRef> for CoreTypeRef {
    fn from(value: stdlib_header::CoreTypeRef) -> Self {
        match value {
            stdlib_header::CoreTypeRef::Core(core_type_id) => Self::Core(core_type_id),
            stdlib_header::CoreTypeRef::WithGeneric(this, generics) => {
                Self::WithGeneric(this, generics.into_iter().map(Self::from).collect())
            }
            stdlib_header::CoreTypeRef::Generic(g) => Self::Generic(g),
        }
    }
}

fn map_core_type_ref_back(a: CoreTypeRef) -> stdlib_header::CoreTypeRef {
    match a {
        crate::CoreTypeRef::Core(core_type_id) => stdlib_header::CoreTypeRef::Core(core_type_id),
        crate::CoreTypeRef::WithGeneric(this, generics) => stdlib_header::CoreTypeRef::WithGeneric(
            this,
            generics.into_iter().map(map_core_type_ref_back).collect(),
        ),
        crate::CoreTypeRef::Generic(g) => stdlib_header::CoreTypeRef::Generic(g),
    }
}

mod core_type_ref_serde {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        value: &stdlib_header::CoreTypeRef,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        crate::CoreTypeRef::serialize(&crate::CoreTypeRef::from(value.clone()), s)
    }
    pub fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        d: D,
    ) -> Result<stdlib_header::CoreTypeRef, D::Error> {
        crate::CoreTypeRef::deserialize(d).map(crate::map_core_type_ref_back)
    }
}

mod vec_core_type_ref_serde {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        value: &Vec<stdlib_header::CoreTypeRef>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        Vec::<crate::CoreTypeRef>::serialize(&value.iter().map(|x| x.clone().into()).collect(), s)
    }
    pub fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<stdlib_header::CoreTypeRef>, D::Error> {
        Vec::<crate::CoreTypeRef>::deserialize(d)
            .map(|x| x.into_iter().map(crate::map_core_type_ref_back).collect())
    }
}

mod option_core_type_ref_serde {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        value: &Option<stdlib_header::CoreTypeRef>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        Option::<crate::CoreTypeRef>::serialize(&value.as_ref().map(|x| x.clone().into()), s)
    }
    pub fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        d: D,
    ) -> Result<Option<stdlib_header::CoreTypeRef>, D::Error> {
        Option::<crate::CoreTypeRef>::deserialize(d).map(|x| x.map(crate::map_core_type_ref_back))
    }
}

pub trait GetCoreTypeInfo: Sized {
    fn get_core_type_info(self) -> fn() -> CoreTypeInfo;
}
impl GetCoreTypeInfo for CoreTypeId {
    fn get_core_type_info(self) -> fn() -> CoreTypeInfo {
        macro aider($($n:ident)*) {
            match self {
                $(
                    Self::$n => $crate::definitions::$n,
                )*
            }
        }

        aider!(
            System_Object
            System_ValueType

            System_Void

            System_Nullable_1

            System_Boolean

            System_UInt8
            System_UInt16
            System_UInt32
            System_UInt64
            System_USize

            System_Int8
            System_Int16
            System_Int32
            System_Int64
            System_ISize

            System_Char

            System_Pointer

            System_NonPurusCallConfiguration
            System_NonPurusCallType

            System_DynamicLibrary

            System_Array_1
            System_String
            System_LargeString

            System_Environment

            System_Exception
            System_InvalidEnumException
            System_Win32Exception
            System_ErrnoException
            System_DlErrorException
        )
    }
}

pub fn get_all_core_type_info() -> Vec<CoreTypeInfo> {
    let mut result = Vec::new();
    for x in CoreTypeId::ALL_VARIANTS {
        result.push(x.get_core_type_info()());
    }
    result
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CoreTypeKind {
    Class,
    Struct,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct GenericCount {
    pub count: u32,
    pub is_infinite: bool,
}
