#![feature(iterator_try_collect)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]
#![feature(const_default)]
#![feature(derive_const)]

mod visibility;
pub use visibility::Visibility;

mod field;
pub use field::{FieldAttr, FieldImplementationFlags};

mod method;
pub use method::{
    CallConvention, MethodAttr, MethodImplementationFlags, ParameterAttr,
    ParameterImplementationFlags,
};

mod ty;
pub use ty::{
    ClassImplementationFlags, InterfaceImplementationFlags, StructImplementationFlags, TypeAttr,
    TypeSpecificAttr,
};
