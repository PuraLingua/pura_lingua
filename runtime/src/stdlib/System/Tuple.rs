#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use stdlib_header::CoreTypeId;

    use crate::{
        memory::GetLayoutOptions, stdlib::CoreTypeIdExt, type_system::class::Class,
        value::managed_reference::ManagedReference,
    };

    #[test]
    fn tuple_test() {
        #[repr(C)]
        struct Tuple1 {
            _0: u32,
            _1: ManagedReference<Class>,
            _2: u8,
        }

        let tuple_t = CoreTypeId::System_Tuple
            .global_type_handle()
            .unwrap_struct();

        let tuple_1_t = unsafe {
            tuple_t.as_ref().instantiate(&[
                CoreTypeId::System_UInt32.global_type_handle().into(),
                CoreTypeId::System_String.global_type_handle().into(),
                CoreTypeId::System_UInt8.global_type_handle().into(),
            ])
        };

        let tuple_1_mt = unsafe { tuple_1_t.as_ref().method_table_ref() };

        assert_eq!(
            Layout::new::<Tuple1>(),
            tuple_1_mt.mem_layout(GetLayoutOptions {
                prefer_cached: false,
                discard_calculated_layout: false,
            })
        );
    }
}
