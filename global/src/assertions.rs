use std::{alloc::Layout, marker::PhantomData};

use _sealed::AssertionSealed;

#[derive_const(Clone)]
#[derive(Copy, Debug)]
pub struct ConstAssert<const BOOL: bool>;

mod _sealed {
    use super::*;

    pub trait AssertionSealed {}
    impl<const BOOL: bool> AssertionSealed for ConstAssert<BOOL> {}
    impl<T1, T2> AssertionSealed for LayoutEq<T1, T2> {}
    impl<T1, T2> AssertionSealed for And<T1, T2> {}
    impl<T> AssertionSealed for Not<T> {}
}

pub trait SuccessAssert: AssertionSealed {}
pub trait FailureAssert: AssertionSealed {}

impl<T: SuccessAssert> !FailureAssert for T {}
impl<T: FailureAssert> !SuccessAssert for T {}

impl SuccessAssert for ConstAssert<true> {}
impl FailureAssert for ConstAssert<false> {}

pub struct LayoutEq<T1, T2>(PhantomData<(T1, T2)>);

#[inline(always)]
const fn layout_eq<T1, T2>() -> bool {
    let layout1 = Layout::new::<T1>();
    let layout2 = Layout::new::<T2>();
    (layout1.size() == layout2.size()) && (layout1.align() == layout2.align())
}

impl<T1, T2> SuccessAssert for LayoutEq<T1, T2> where
    ConstAssert<{ layout_eq::<T1, T2>() }>: SuccessAssert
{
}

pub struct And<T1, T2>(PhantomData<(T1, T2)>);

impl<T1, T2> SuccessAssert for And<T1, T2>
where
    T1: SuccessAssert,
    T2: SuccessAssert,
{
}

pub struct Not<T>(PhantomData<T>);

impl<T> SuccessAssert for Not<T> where T: FailureAssert {}
