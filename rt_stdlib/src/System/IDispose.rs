use proc_macros::define_core_interface;

use crate::{CoreTypeId, CoreTypeRef};

define_core_interface! {
    #[Public {}] assembly
    System_IDispose "System::IDispose" =>

    #methods:
    [
        #[Public {}] Dispose "Dispose" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ]
}
