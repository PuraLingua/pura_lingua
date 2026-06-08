use std::{
    ptr::NonNull,
    sync::{
        Arc,
        nonpoison::{Condvar, Mutex},
    },
};

use stdlib_header::{CoreTypeId, System::ThreadLocal_1};

use crate::{
    memory::ThreadSafeNonNull,
    stdlib::CoreTypeIdExt,
    test_utils::g_core_class,
    type_system::class::Class,
    virtual_machine::{EnsureGlobalVirtualMachineInitialized, cpu_manager::CpuID},
};

#[test]
fn tls_support() {
    EnsureGlobalVirtualMachineInitialized();

    let mut cpu = CpuID::new_write_global();

    let ThreadLocal_1: &Class = unsafe { g_core_class!(System_ThreadLocal_1).as_ref() };

    let instantiated = ThreadLocal_1.instantiate(&[CoreTypeId::System_UInt64.global_type_handle()]);

    let mt: &crate::type_system::method_table::MethodTable<Class> =
        unsafe { instantiated.as_ref().method_table_ref() };

    let Get: ThreadSafeNonNull<crate::type_system::method::Method<Class>> =
        ThreadSafeNonNull::new(*mt.get_method(ThreadLocal_1::MethodId::Get as u32).unwrap());
    let Set: ThreadSafeNonNull<crate::type_system::method::Method<Class>> =
        ThreadSafeNonNull::new(*mt.get_method(ThreadLocal_1::MethodId::Set as u32).unwrap());

    let var = cpu
        .new_object(
            instantiated,
            &(ThreadLocal_1::MethodId::Constructor.into()),
            &[],
        )
        .unwrap();

    let synchronizer = Arc::new((Mutex::new(false), Condvar::new()));
    let synchronizer2 = Arc::clone(&synchronizer);

    let thread1 = std::thread::spawn(move || {
        let (lock, cvar) = &*synchronizer2;
        let mut started = lock.lock();

        let Get: &crate::type_system::method::Method<Class> = unsafe { Get.as_ref() };
        let Set: &crate::type_system::method::Method<Class> = unsafe { Set.as_ref() };

        let mut cpu = CpuID::new_write_global();
        let data = 100u64;
        Set.typed_res_call::<()>(
            &mut cpu,
            Some(NonNull::from_ref(&var).cast()),
            &[(&raw const data).cast_mut().cast()],
        );

        *started = true;
        cvar.notify_one();

        let data_got: u64 = Get.typed_res_call(&mut cpu, Some(NonNull::from_ref(&var).cast()), &[]);

        assert_eq!(data, data_got);
    });

    let thread2 = std::thread::spawn(move || {
        let (lock, cvar) = &*synchronizer;

        let Get: &crate::type_system::method::Method<Class> = unsafe { Get.as_ref() };
        let Set: &crate::type_system::method::Method<Class> = unsafe { Set.as_ref() };

        let mut started = lock.lock();
        while !*started {
            cvar.wait(&mut started);
        }

        let mut cpu = CpuID::new_write_global();
        let data = 50u64;
        Set.typed_res_call::<()>(
            &mut cpu,
            Some(NonNull::from_ref(&var).cast()),
            &[(&raw const data).cast_mut().cast()],
        );

        let data_got: u64 = Get.typed_res_call(&mut cpu, Some(NonNull::from_ref(&var).cast()), &[]);

        assert_eq!(data, data_got);
    });

    thread1.join().unwrap();
    thread2.join().unwrap();
}
