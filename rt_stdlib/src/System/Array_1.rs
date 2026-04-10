use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_Array_1 1 "System::Array`1" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[override Some(super::Object::MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Private {}] GetPointerOfIndex (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[
            Public {}
            CoreTypeRef::Core(CoreTypeId::System_Object),
            CoreTypeRef::Core(CoreTypeId::System_USize), // arg 0
            CoreTypeRef::Core(CoreTypeId::System_Pointer),
            CoreTypeRef::Core(CoreTypeId::System_USize), // Size of T
            CoreTypeRef::Generic(0),
        ] get_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Generic(0);
        #[
            Public {}
            CoreTypeRef::Core(CoreTypeId::System_Object), // this
            CoreTypeRef::Core(CoreTypeId::System_USize), // arg 0
            CoreTypeRef::Generic(0), // arg 1
            CoreTypeRef::Core(CoreTypeId::System_USize), // size of T
            CoreTypeRef::Core(CoreTypeId::System_Pointer), // pointer of result
        ] set_Index (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
            #[{}] CoreTypeRef::Generic(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
