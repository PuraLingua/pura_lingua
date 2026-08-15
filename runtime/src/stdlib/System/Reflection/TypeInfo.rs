use stdlib_header::System::Reflection::TypeInfo::FieldId;

use crate::{
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

fn get_Name(
    _cpu: &mut CPU,
    _method: &Method<Class>,
    this: &mut ManagedReference<Class>,
) -> ManagedReference<Class> {
    this.const_access::<FieldAccessor<Class>>()
        .read_typed_field(FieldId::Name as _, Default::default())
        .unwrap()
}

super::_define_class!(
    fn load(assembly, mt, method_info)
    TypeInfo
#methods(TMethodId):
    GetName => crate::stdlib::System::common_new_method!(mt TMethodId GetName get_Name);
#static_methods(TStaticMethodId):
    StaticConstructor => crate::stdlib::System::default_sctor!(mt TStaticMethodId);
);

#[cfg(test)]
mod tests {
    use stdlib_header::CoreTypeId;
    use widestring::u16str;

    use crate::{
        type_system::reflection_info_container::IReflect, value::managed_reference::StringAccessor,
        virtual_machine::global_vm,
    };

    use super::*;

    #[test]
    fn test_reflection_type() {
        let ty = global_vm()
            .assembly_manager()
            .get_core_type(CoreTypeId::System_Object);
        ty.__reflect_update();

        assert_eq!(
            ty.__get_reflect_value()
                .const_access::<FieldAccessor<Class>>()
                .read_typed_field::<ManagedReference<Class>>(FieldId::Name as _, Default::default())
                .unwrap()
                .access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap(),
            u16str!("System::Object")
        );

        let ty = global_vm()
            .assembly_manager()
            .get_core_type(CoreTypeId::System_Int64);
        ty.__reflect_update();

        assert_eq!(
            ty.__get_reflect_value()
                .const_access::<FieldAccessor<Class>>()
                .read_typed_field::<ManagedReference<Class>>(FieldId::Name as _, Default::default())
                .unwrap()
                .access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap(),
            u16str!("System::Int64")
        );

        let ty = global_vm()
            .assembly_manager()
            .get_core_type(CoreTypeId::System_IDispose);
        ty.__reflect_update();

        assert_eq!(
            ty.__get_reflect_value()
                .const_access::<FieldAccessor<Class>>()
                .read_typed_field::<ManagedReference<Class>>(FieldId::Name as _, Default::default())
                .unwrap()
                .access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap(),
            u16str!("System::IDispose")
        );
    }
}
