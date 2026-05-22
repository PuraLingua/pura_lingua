use crate::CoreTypeId;

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_DlErrorException Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields:

    #methods of super::Exception::MethodId:
    [] []
}
