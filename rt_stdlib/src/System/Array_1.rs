use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_Array_1 1 Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[override Some(super::Object::MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Public {}] GetReference (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[Public {UseReturnBuffer}] get_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::TypeGeneric(0);
        #[
            Public {UseReturnBuffer}
            CoreTypeRef::Core(CoreTypeId::System_Object), // this
            CoreTypeRef::Core(CoreTypeId::System_USize), // arg 0
            CoreTypeRef::TypeGeneric(0), // arg 1
            CoreTypeRef::Core(CoreTypeId::System_USize), // size of T
            CoreTypeRef::Core(CoreTypeId::System_Pointer), // pointer of result
        ] set_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
            #[{}] CoreTypeRef::TypeGeneric(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
