use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_InvalidEnumException "System::InvalidEnumException"
    Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields:

    #methods of super::Exception::MethodId:
    [
        #[Public {}] Constructor_String_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
