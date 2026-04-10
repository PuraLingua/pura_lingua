use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_Object "System::Object" =>
    #fields:

    #methods:
    [
        #[Public {}] Destructor "~" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
    ] []
}
