#![feature(decl_macro)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_try)]

pub extern crate binary_core;
pub extern crate proc_macros;

pub mod item_token;

pub mod assembly;
pub mod custom_attribute;
pub mod ty;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::item_token::{
        ItemToken, ItemTokenBuilder, ItemType, MethodToken, MethodTokenBuilder, MethodType,
        TypeToken, TypeTokenBuilder, TypeType,
    };

    pub use crate::assembly::Assembly;
    pub use crate::custom_attribute::{CustomAttribute, CustomAttributeValue, Integer};
    pub use crate::ty::{ClassDef, StructDef, TypeDef, TypeRef, TypeSpec};
    pub use binary_core::{BinaryResult, Error};
}
