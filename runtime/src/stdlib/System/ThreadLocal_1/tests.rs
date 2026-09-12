use std::{
    ptr::NonNull,
    sync::{
        Arc,
        nonpoison::{Condvar, Mutex},
    },
};

use stdlib_header::{CoreTypeId, System::ThreadLocal_1};

use crate::{
    memory::ThreadSafeNonNull, stdlib::CoreTypeIdExt, test_utils::core_class_in,
    virtual_machine::create_vm_on_stack,
};

#[test]
fn tls_support() {
    create_vm_on_stack!(vm);
    let vmr = unsafe { (&raw const vm).as_ref_unchecked() };

    let mut cpu = vm.add_write_cpu();

    let ThreadLocal_1 = unsafe { core_class_in!(System_ThreadLocal_1 in vm).as_ref() };

    let instantiated = ThreadLocal_1.instantiate(&[CoreTypeId::System_UInt64.global_type_handle()]);

    let mt = unsafe { instantiated.as_ref().method_table_ref() };

    let Get = ThreadSafeNonNull::new(*mt.get_method(ThreadLocal_1::MethodId::Get as u32).unwrap());
    let Set = ThreadSafeNonNull::new(*mt.get_method(ThreadLocal_1::MethodId::Set as u32).unwrap());

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

        let Get = unsafe { Get.as_ref() };
        let Set = unsafe { Set.as_ref() };

        let mut cpu = vmr.add_write_cpu();
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

        let Get = unsafe { Get.as_ref() };
        let Set = unsafe { Set.as_ref() };

        let mut started = lock.lock();
        while !*started {
            cvar.wait(&mut started);
        }

        let mut cpu = vmr.add_write_cpu();
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
