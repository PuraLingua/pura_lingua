use std::{ptr::NonNull, range::Range, sync::Mutex};

use global::getset::{CopyGetters, Getters};

use crate::type_system::{
    class::Class,
    get_traits::{GetAssemblyRef, GetTypeVars},
    method::{Method, MethodRef},
    type_handle::{
        MaybeUnloadedTypeHandle, MethodGenericResolver, NonGenericTypeHandle,
        NonGenericTypeHandleKind, TypeHandle,
    },
};

pub struct ExceptionTable<T> {
    method: NonNull<Method<T>>,
    entries: Vec<ExceptionTableEntry>,
}

impl<T> Clone for ExceptionTable<T> {
    fn clone(&self) -> Self {
        Self {
            method: self.method,
            entries: self.entries.clone(),
        }
    }
}

struct ExceptionTableNew<T>(core::marker::PhantomData<T>);

impl<T> const FnOnce<(&Method<T>,)> for ExceptionTableNew<T> {
    type Output = ExceptionTable<T>;
    extern "rust-call" fn call_once(self, args: (&Method<T>,)) -> Self::Output {
        ExceptionTable::new(NonNull::from_ref(args.0))
    }
}

impl<T> ExceptionTable<T> {
    pub const fn new(method: NonNull<Method<T>>) -> Self {
        Self {
            method,
            entries: vec![],
        }
    }
    /// Used for [`Method::new`]
    ///
    /// [`Method::new`]: crate::type_system::method::Method::new
    pub const fn gen_new() -> impl FnOnce(&Method<T>) -> Self {
        ExceptionTableNew(core::marker::PhantomData)
    }
    pub(crate) fn reset_method_ptr(&mut self, method: NonNull<Method<T>>) {
        self.method = method;
    }
    pub fn resort(&mut self) {
        self.entries.sort_by(|a, b| {
            match b
                .range
                .start
                .cmp(&a.range.start)
                .then(a.range.end.cmp(&b.range.end))
            {
                std::cmp::Ordering::Equal
                    if let Some(a_type) = *a.exception_type_cache.lock().unwrap()
                        && let Some(b_type) = *b.exception_type_cache.lock().unwrap() =>
                {
                    let a_mt = unsafe { a_type.as_ref() }.method_table_ref();
                    let b_mt = unsafe { b_type.as_ref() }.method_table_ref();

                    if a_mt.is_inherited_from(b_mt) {
                        std::cmp::Ordering::Less
                    } else if b_mt.is_inherited_from(a_mt) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                std::cmp::Ordering::Equal
                    if let MaybeUnloadedTypeHandle::Loaded(TypeHandle::Class(a_type)) =
                        a.exception_type
                        && let MaybeUnloadedTypeHandle::Loaded(TypeHandle::Class(b_type)) =
                            b.exception_type =>
                {
                    let a_mt = unsafe { a_type.as_ref() }.method_table_ref();
                    let b_mt = unsafe { b_type.as_ref() }.method_table_ref();

                    if a_mt.is_inherited_from(b_mt) {
                        std::cmp::Ordering::Less
                    } else if b_mt.is_inherited_from(a_mt) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                x => x,
            }
        });
    }

    pub fn push(&mut self, entry: ExceptionTableEntry) {
        self.entries.push(entry);
        self.resort();
    }

    pub fn get_for(&self, pc: usize) -> impl Iterator<Item = &ExceptionTableEntry> {
        ExceptionTableLookupFor::new(self, pc)
    }
}

struct ExceptionTableLookupFor<'a, T> {
    this: &'a ExceptionTable<T>,
    middle: usize,
    pc: usize,
}

impl<'a, T> ExceptionTableLookupFor<'a, T> {
    pub fn new(this: &'a ExceptionTable<T>, pc: usize) -> Self {
        Self {
            this,
            middle: this.entries.len().div_ceil(2),
            pc,
        }
    }
}

impl<'a, T> Iterator for ExceptionTableLookupFor<'a, T> {
    type Item = &'a ExceptionTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.this.entries.is_empty() {
            return None;
        }
        loop {
            let x = unsafe { self.this.entries.get_unchecked(self.middle) };
            match x.check(self.pc) {
                std::cmp::Ordering::Less => {
                    let next_middle = (self.this.entries.len() - self.middle).div_ceil(2);
                    let overflowed;
                    (self.middle, overflowed) = self.middle.overflowing_add(next_middle);
                    if overflowed || self.middle >= self.this.entries.len() {
                        return None;
                    }
                }
                std::cmp::Ordering::Equal => return Some(x),
                std::cmp::Ordering::Greater => {
                    let next_middle = (self.this.entries.len() - self.middle).div_ceil(2);
                    let overflowed;
                    (self.middle, overflowed) = self.middle.overflowing_sub(next_middle);
                    if overflowed {
                        return None;
                    }
                }
            }
        }
    }
}

impl<TType, A> Extend<A> for ExceptionTable<TType>
where
    Vec<ExceptionTableEntry>: Extend<A>,
{
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.entries.extend(iter);
        self.resort();
    }
    fn extend_one(&mut self, item: A) {
        self.entries.extend_one(item);
        self.resort();
    }
    fn extend_reserve(&mut self, additional: usize) {
        self.entries.extend_reserve(additional);
    }
}

#[derive(Getters, CopyGetters)]
pub struct ExceptionTableEntry {
    range: Range<u64>,
    exception_type: MaybeUnloadedTypeHandle,
    exception_type_cache: Mutex<Option<NonNull<Class>>>,

    filter: Option<(MaybeUnloadedTypeHandle, MethodRef)>,
    filter_cache: Mutex<Option<(NonGenericTypeHandleKind, NonNull<Method<()>>)>>,

    #[getset(get_copy = "pub const")]
    catch: Range<u64>,
    #[getset(get_copy = "pub const")]
    finally: Option<Range<u64>>,
    #[getset(get_copy = "pub const")]
    fault: Option<Range<u64>>,
}

impl Clone for ExceptionTableEntry {
    fn clone(&self) -> Self {
        Self {
            range: self.range,
            exception_type: self.exception_type.clone(),
            exception_type_cache: Mutex::new(None),

            filter: self.filter.clone(),
            filter_cache: Mutex::new(None),

            catch: self.catch.clone(),
            finally: self.finally.clone(),
            fault: self.fault.clone(),
        }
    }
}

impl ExceptionTableEntry {
    pub const fn new<TExceptionType: [const] Into<MaybeUnloadedTypeHandle>>(
        range: Range<u64>,
        exception_type: TExceptionType,

        filter: Option<(MaybeUnloadedTypeHandle, MethodRef)>,

        catch: Range<u64>,
        finally: Option<Range<u64>>,
        fault: Option<Range<u64>>,
    ) -> Self {
        Self {
            range,
            exception_type: exception_type.into(),
            exception_type_cache: Mutex::new(None),

            filter,
            filter_cache: Mutex::new(None),

            catch,
            finally,
            fault,
        }
    }

    const fn check(&self, pc: usize) -> std::cmp::Ordering {
        if self.can_handle(pc) {
            std::cmp::Ordering::Equal
        } else if (pc as u64) > self.range.end {
            std::cmp::Ordering::Greater
        } else if (pc as u64) < self.range.start {
            std::cmp::Ordering::Less
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }
    pub fn get_exception_type<T: GetAssemblyRef + GetTypeVars>(
        &self,
        method: &Method<T>,
    ) -> Option<NonNull<Class>> {
        let mut exception_type_cache = self.exception_type_cache.lock().unwrap();
        if let Some(cache) = *exception_type_cache {
            return Some(cache);
        }
        let ty = self.exception_type.load_with_generic_resolver(
            method
                .require_method_table_ref()
                .ty_ref()
                .__get_assembly_ref()
                .manager_ref(),
            MethodGenericResolver::new(method),
        )?;
        if let TypeHandle::Class(cl) = ty {
            *exception_type_cache = Some(cl);
            return Some(cl);
        }
        match ty.get_non_generic_with_method(method).unwrap() {
            NonGenericTypeHandle::Class(class) => Some(class),
            _ => None,
        }
    }
    pub fn get_filter<T: GetAssemblyRef + GetTypeVars>(
        &self,
        method: &Method<T>,
    ) -> Option<(NonGenericTypeHandleKind, NonNull<Method<()>>)> {
        if let Some((ty, m_ref)) = self.filter.as_ref() {
            let mut cache = self.filter_cache.lock().unwrap();
            if let Some(cache) = *cache {
                return Some(cache);
            }
            let ty = ty.load_with_generic_resolver(
                method
                    .require_method_table_ref()
                    .ty_ref()
                    .__get_assembly_ref()
                    .manager_ref(),
                MethodGenericResolver::new(method),
            )?;
            match ty.get_non_generic_with_method(method).unwrap() {
                NonGenericTypeHandle::Class(cl) => {
                    let mt_ref = unsafe { cl.as_ref() }.method_table_ref();
                    let method = mt_ref.get_method_by_ref(m_ref)?;
                    let data = (NonGenericTypeHandleKind::Class, method.cast());
                    *cache = Some(data);
                    Some(data)
                }
                NonGenericTypeHandle::Struct(st) => {
                    let mt_ref = unsafe { st.as_ref() }.method_table_ref();
                    let method = mt_ref.get_method_by_ref(m_ref)?;
                    let data = (NonGenericTypeHandleKind::Struct, method.cast());
                    *cache = Some(data);
                    Some(data)
                }
                NonGenericTypeHandle::Interface(_) => None,
            }
        } else {
            None
        }
    }
    pub const fn can_handle(&self, pc: usize) -> bool {
        self.range.contains(&(pc as u64))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::g_core_type;

    use super::*;

    fn generate_entry(r: std::ops::Range<u64>) -> ExceptionTableEntry {
        ExceptionTableEntry::new(
            r.into(),
            g_core_type!(System_Void),
            None,
            Default::default(),
            None,
            None,
        )
    }

    #[test]
    fn test_search_exception_entry() {
        let mut current = 5;
        let table: ExceptionTable<()> = std::iter::repeat_with(|| {
            let e = generate_entry(current..current + 10);
            current += 9;
            e
        })
        .take(10)
        .fold(
            ExceptionTable::new(NonNull::dangling()),
            |mut table, entry| {
                table.push(entry);
                table
            },
        );

        assert!(table.get_for(9).next().is_some());
        assert!(table.get_for(22).next().is_some());
        assert!(table.get_for(6).next().is_some());

        assert!(table.get_for(3).next().is_none());
        assert!(table.get_for(99).next().is_none());
    }
}
