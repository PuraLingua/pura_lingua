use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_Reflection_TypeInfo Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::super::Object::FieldId:
    #[Private {}] Name "_Name" => CoreTypeRef::Core(CoreTypeId::System_String);

    #methods of super::super::Object::MethodId:
    [
        #[Public {}] GetName "get$Name" () -> CoreTypeRef::Core(CoreTypeId::System_String);
    ] []
}
