use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_UInt8 "System::UInt8" =>
    [None]
    #fields:
    #methods:
    [] [
        #[Public {Static}] ToString (
            #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
        ) -> CoreTypeRef::Core(CoreTypeId::System_String);
    ]
}
