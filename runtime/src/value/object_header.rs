use std::{hint::unlikely, sync::RwLock, thread::ThreadId};

use bitfields::{FromBits, IntoBits, bitfield};
use parking_lot::{RawMutex, lock_api::RawMutex as _};

#[bitfield(u64, new = false)]
pub struct ObjectHeader {
    is_marked: bool,
    is_static: bool,
    #[bits(6)]
    _pad1: u8,
    #[bits(32)]
    sync: Sync,
    #[bits(24)]
    _pad: u32,
}

impl ObjectHeader {
    pub fn new(is_static: bool) -> Self {
        ObjectHeaderBuilder::new()
            .with_is_marked(false)
            .with_is_static(is_static)
            .with_sync(Sync::new())
            .build()
    }
}

#[derive(Clone, Copy)]
pub union Sync {
    thin: ThinSync,
    fat: FatSync,
}

const _: () = {
    use global::assertions::{And, LayoutEq, SuccessAssert};
    fn _assert()
    where
        And<And<LayoutEq<Sync, u32>, LayoutEq<ThinSync, Sync>>, LayoutEq<FatSync, Sync>>:
            SuccessAssert,
    {
    }
};

impl Sync {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let thin = ThinSync::new(std::thread::current_id());
        unsafe { std::mem::transmute(thin) }
    }

    pub fn to_fat(&mut self) {
        if unlikely(unsafe { !self.thin.is_thin() }) {
            return;
        }

        let id = unsafe { self.thin.id() };
        let _g = G_SYNC_BLOCKS_LOCK.write().unwrap();
        let index;
        if let Some((i, sync_block)) =
            unsafe { G_SYNC_BLOCKS.iter_mut().enumerate().find(|x| !x.1.is_valid) }
        {
            index = i;
            sync_block.is_valid = true;
            sync_block.id = id;
            sync_block.lock = RawMutex::INIT;
        } else {
            index = unsafe { G_SYNC_BLOCKS.len() };
            unsafe {
                G_SYNC_BLOCKS.push(SyncBlock {
                    is_valid: true,
                    id,
                    lock: RawMutex::INIT,
                })
            };
        }

        self.fat = FatSyncBuilder::new().with_index(index).build();
    }

    pub fn lock(&mut self) {
        if unsafe { self.thin.is_thin() } {
            self.to_fat();
        }

        unsafe {
            let sync_block = G_SYNC_BLOCKS.get(self.fat.index()).unwrap();
            sync_block.lock.lock();
        }
    }

    pub fn unlock(&mut self) {
        if unsafe { self.thin.is_thin() }
            || unsafe { self.fat.get_block().is_none_or(|x| !x.lock.is_locked()) }
        {
            return;
        }

        unsafe {
            let sync_block = G_SYNC_BLOCKS.get(self.fat.index()).unwrap();
            sync_block.lock.unlock();
        }
    }

    pub fn destroy(&self) {
        if unsafe { self.thin.is_thin() } {
            return;
        }

        unsafe { self.fat.destroy() }
    }
}

impl const FromBits for Sync {
    type Number = u32;

    fn from_bits(bits: Self::Number) -> Self {
        unsafe { std::mem::transmute(bits) }
    }
}

impl const IntoBits for Sync {
    type Number = u32;

    fn into_bits(self) -> Self::Number {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Clone, Copy)]
#[bitfield(u32, from_endian = little, into_endian = little, order = lsb, new = false, builder = false)]
pub struct ThinSync {
    #[bits(31)]
    id: ThreadId,
    #[bits(default = true)]
    is_thin: bool,
}

impl ThinSync {
    pub fn new(id: ThreadId) -> Self {
        let mut this = Self(0);
        this.set_id(id);
        this.set_is_thin(true);

        this
    }
}

#[derive(Clone, Copy)]
#[bitfield(u32)]
pub struct FatSync {
    #[bits(31)]
    index: usize,
    #[bits(default = false)]
    is_thin: bool,
}

impl FatSync {
    pub fn get_block(&self) -> Option<&SyncBlock> {
        unsafe { G_SYNC_BLOCKS.get(self.index()).filter(|x| x.is_valid) }
    }

    pub fn get_block_mut(&self) -> Option<&mut SyncBlock> {
        unsafe { G_SYNC_BLOCKS.get_mut(self.index()).filter(|x| x.is_valid) }
    }

    pub fn destroy(&self) {
        let Some(block) = self.get_block_mut() else {
            return; // It has been destroyed
        };
        block.is_valid = false;
    }
}

pub struct SyncBlock {
    is_valid: bool,
    id: ThreadId,

    lock: RawMutex,
}

impl Default for SyncBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncBlock {
    #[inline]
    pub fn new() -> Self {
        Self::with_thread_id(std::thread::current_id())
    }

    #[inline]
    pub const fn with_thread_id(id: ThreadId) -> Self {
        Self {
            is_valid: true,
            id,
            lock: RawMutex::INIT,
        }
    }
}

static G_SYNC_BLOCKS_LOCK: RwLock<()> = RwLock::new(());
static mut G_SYNC_BLOCKS: Vec<SyncBlock> = Vec::new();
