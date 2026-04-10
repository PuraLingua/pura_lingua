use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Int16 "System::Int16" =>
    [None]
    #fields:
    #methods:
    [] [
        #[Public {Static}] ToString (
            #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_Int16)
        ) -> CoreTypeRef::Core(CoreTypeId::System_String);
    ]
}
