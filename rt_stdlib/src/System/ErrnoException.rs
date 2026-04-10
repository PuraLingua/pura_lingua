use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_ErrnoException "System::ErrnoException"
    Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields of super::Exception::FieldId:
    #[Public {}] Code "_Code" => CoreTypeId::System_Int32.into();

    #methods of super::Exception::MethodId:
    [
        #[Public {}] Constructor ".ctor" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_I32 ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Int32)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
