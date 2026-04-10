use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_UInt16 "System::UInt16" =>
    [None]
    #fields:
    #methods:
    [] [
        #[Public {Static}] ToString (
            #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt16)
        ) -> CoreTypeRef::Core(CoreTypeId::System_String);
    ]
}
