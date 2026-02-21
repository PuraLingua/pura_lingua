use std::{
    alloc::{Allocator, Layout},
    ptr::{Alignment, NonNull},
    sync::{
        MappedRwLockReadGuard, MappedRwLockWriteGuard, OnceLock, RwLock, RwLockReadGuard,
        RwLockWriteGuard,
    },
};

use global::ThreadSafe;
use slab::Slab;

#[repr(C)]
#[derive(ThreadSafe)]
pub struct ResourcePtr {
    pub ptr: NonNull<u8>,
    pub vtable: &'static ResourceVTable,
}

#[repr(C)]
pub struct ResourceVTable {
    pub size: usize,
    pub align: Alignment,
    pub dropper: extern "system" fn(NonNull<u8>),
}

pub struct BoxedResource<A: Allocator = std::alloc::Global>(ResourcePtr, A);

impl<A: Allocator> Drop for BoxedResource<A> {
    fn drop(&mut self) {
        (self.0.vtable.dropper)(self.0.ptr);
        unsafe {
            self.1.deallocate(
                self.0.ptr,
                Layout::from_size_alignment_unchecked(self.0.vtable.size, self.0.vtable.align),
            );
        }
    }
}

impl BoxedResource {
    pub fn new<T>(val: T) -> Self {
        Self::new_in(val, std::alloc::Global)
    }
}

impl<A: Allocator> BoxedResource<A> {
    pub fn new_in<T>(val: T, allocator: A) -> Self {
        let layout = Layout::new::<T>();
        match Self::try_new_in(val, allocator) {
            Ok(this) => this,
            Err(_) => std::alloc::handle_alloc_error(layout),
        }
    }

    pub fn try_new_in<T>(val: T, allocator: A) -> Result<Self, std::alloc::AllocError> {
        static VTABLE: OnceLock<ResourceVTable> = OnceLock::new();
        extern "system" fn dropper<T>(ptr: NonNull<u8>) {
            unsafe {
                ptr.cast::<T>().drop_in_place();
            }
        }
        let vtable = VTABLE.get_or_init(|| ResourceVTable {
            size: size_of::<T>(),
            align: Alignment::of::<T>(),
            dropper: dropper::<T>,
        });
        let ptr = allocator.allocate(Layout::new::<T>())?.as_non_null_ptr();
        unsafe {
            ptr.cast::<T>().copy_from(NonNull::from_ref(&val), 1);
        }
        std::mem::forget(val);
        Ok(Self(ResourcePtr { ptr, vtable }, allocator))
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.0.ptr.as_ptr().cast_const().cast(), self.0.vtable.size)
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.0.ptr.as_ptr().cast(), self.0.vtable.size) }
    }
}

#[repr(transparent)]
#[derive_const(Clone)]
pub struct ResourceHandle(usize);

impl ResourceHandle {
    pub fn free(self) {
        let mut resources = RESOURCES.write().unwrap();
        drop(resources.remove(self.0));
    }

    pub fn read(&self) -> MappedRwLockReadGuard<'static, BoxedResource> {
        RwLockReadGuard::map(RESOURCES.read().unwrap(), |x| unsafe {
            x.get_unchecked(self.0)
        })
    }
    pub fn write(&self) -> MappedRwLockWriteGuard<'static, BoxedResource> {
        RwLockWriteGuard::map(RESOURCES.write().unwrap(), |x| unsafe {
            x.get_unchecked_mut(self.0)
        })
    }
}

static RESOURCES: RwLock<Slab<BoxedResource>> = RwLock::new(Slab::new());

pub fn add_resource(resource: BoxedResource) -> ResourceHandle {
    let mut resources = RESOURCES.write().unwrap();
    ResourceHandle(resources.insert(resource))
}

pub struct ResourceManager {}

impl ResourceManager {
    pub fn add_resource(&self, resource: BoxedResource) -> ResourceHandle {
        add_resource(resource)
    }
}
