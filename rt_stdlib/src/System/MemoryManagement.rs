use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class!(
    #[Public {}] assembly
    System_MemoryManagement Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:

    #methods of super::Object::MethodId:
    [] [
        #[Public {}] FreeObject "FreeObject"(
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Object)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ]
);
