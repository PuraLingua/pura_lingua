use crate::CoreTypeId;

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Pointer "System::Pointer" =>
    #fields:
    #[Public {Static}]
    Null "Null" => CoreTypeId::System_Pointer.into();

    #methods:
    [] []
}
