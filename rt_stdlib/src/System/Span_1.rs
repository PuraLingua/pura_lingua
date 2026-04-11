use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Span_1 1 "System::Span`1" =>
    #fields:
    #[Public {}] Data "Data" => CoreTypeRef::Core(CoreTypeId::System_Pointer);
    #[Public {}] Length "Length" => CoreTypeRef::Core(CoreTypeId::System_USize);

    #methods:
    [] []
}
