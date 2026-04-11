use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_DlErrorException "System::DlErrorException"
    Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields:

    #methods of super::Exception::MethodId:
    [
        #[Public {}] Constructor ".ctor" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
