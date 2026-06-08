use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_ThreadLocal_1 1 Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);

        #[Public {}]
        Constructor ".ctor"() -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}]
        GetPointer () -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[Public {UseReturnBuffer}]
        Get () -> CoreTypeRef::TypeGeneric(0);
        #[
            Public {}
            CoreTypeRef::TypeGeneric(0), // value
            CoreTypeRef::Core(CoreTypeId::System_USize), // size of value
            CoreTypeRef::WithGeneric(CoreTypeId::System_ThreadLocal_1, vec![CoreTypeRef::TypeGeneric(0)]), // this
            CoreTypeRef::Core(CoreTypeId::System_Pointer), // pointer to storage
        ]
        Set (
            #[{}] CoreTypeRef::TypeGeneric(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
