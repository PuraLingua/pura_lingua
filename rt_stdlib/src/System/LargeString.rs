use proc_macros::define_core_class;

use crate::{CoreTypeId, CoreTypeRef};

define_core_class! {
    #[Public {}] assembly
    System_LargeString "System::LargeString" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
    ] []
}
