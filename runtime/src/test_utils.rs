pub macro g_core_type($i:ident) {
    $crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
        $crate::stdlib::CoreTypeId::$i.global_type_handle(),
    )
}

pub macro g_core_class($i:ident) {
    $crate::stdlib::CoreTypeId::$i
        .global_type_handle()
        .unwrap_class()
}
