use crate::stdlib::System::{_define_struct, default_sctor};

_define_struct!(
    fn load(assembly, mt, method_info)
    System_Boolean
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
