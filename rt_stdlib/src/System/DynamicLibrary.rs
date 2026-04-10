use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_DynamicLibrary "System::DynamicLibrary" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:
    #[Private {}] Handle "_handle" => CoreTypeId::System_Pointer.into();

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::Destructor as _) Public {}]
        Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] GetSymbol "GetSymbol" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
    ] []
}
