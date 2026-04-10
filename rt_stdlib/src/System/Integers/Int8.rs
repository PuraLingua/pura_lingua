use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Int8 "System::Int8" =>
    [None]
    #fields:
    #methods:
    [] [
        #[Public {Static}] ToString (
            #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int8)
        ) -> CoreTypeRef::Core(CoreTypeId::System_String);
    ]
}
