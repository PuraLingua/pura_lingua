use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_[<NAME>] "System::[<NAME>]" =>
    [None]
    #fields:
    #methods:
    [] [
        #[Public {Static}] ToString (
            #[{ByRef}] CoreTypeRef::Core(CoreTypeId::System_[<NAME>])
        ) -> CoreTypeRef::Core(CoreTypeId::System_String);
    ]
}
