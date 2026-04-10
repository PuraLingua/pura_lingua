use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_String "System::String" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Public {}] get_Length () -> CoreTypeRef::Core(CoreTypeId::System_USize);
        #[Public {}] get_U32Length () -> CoreTypeRef::Core(CoreTypeId::System_UInt32);
    ] []
}
