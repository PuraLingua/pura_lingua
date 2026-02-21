use mem_leak_detector::LeakDetector;

pub macro g_core_type($i:ident) {
    $crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
        <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
            $crate::stdlib::CoreTypeId::$i,
        ),
    )
}

pub macro g_core_class($i:ident) {
    <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
        $crate::stdlib::CoreTypeId::$i,
    )
    .unwrap_class()
}

#[global_allocator]
pub static LEAK_DETECTOR: LeakDetector<std::alloc::System> = LeakDetector::system();
