use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_IndexOutOfRangeException Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields of super::Exception::FieldId:
    #[Public {}] Index "_Index" => CoreTypeId::System_USize.into();
    #[Public {}] Length "_Length" => CoreTypeId::System_USize.into();

    #methods of super::Exception::MethodId:
    [
        #[Public {}] Constructor ".ctor" (
            /* index */ #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
            /* length */ #[{}] CoreTypeRef::Core(CoreTypeId::System_USize)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
