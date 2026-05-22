super::_define_class!(
    fn load(assembly, mt, method_info)
    MethodInfo
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => crate::stdlib::System::default_sctor!(mt TStaticMethodId);
);
