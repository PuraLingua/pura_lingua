use crate::stdlib::System::{_define_class, default_sctor};

_define_class!(
    fn load(assembly, mt, method_info)
    DlErrorException
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
