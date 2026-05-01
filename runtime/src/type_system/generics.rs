use std::{
    fmt::Write,
    ops::{Bound, RangeBounds},
    range::{RangeFrom, RangeToInclusive},
};

use global::getset::Getters;

use crate::type_system::type_handle::MaybeUnloadedTypeHandle;

#[derive(Getters)]
pub struct GenericBounds {
    #[allow(dead_code)]
    pub(crate) implemented_interfaces: Vec<MaybeUnloadedTypeHandle>,
    #[allow(dead_code)]
    pub(crate) parent: Option<MaybeUnloadedTypeHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericCountRequirement {
    AtLeast(RangeFrom<u32>),
    NoMoreThan(RangeToInclusive<u32>),
    Exact(u32),
}

impl const From<stdlib_header::GenericCount> for GenericCountRequirement {
    fn from(value: stdlib_header::GenericCount) -> Self {
        if value.is_infinite {
            Self::AtLeast(RangeFrom { start: value.count })
        } else {
            Self::Exact(value.count)
        }
    }
}

impl const Default for GenericCountRequirement {
    fn default() -> Self {
        Self::Exact(0)
    }
}

impl GenericCountRequirement {
    pub fn decorate(&self, out: &mut String) {
        match self {
            Self::AtLeast(range_from) => {
                let _ = out.write_fmt(format_args!("`{}+", range_from.start));
            }
            Self::NoMoreThan(range_to_inclusive) => {
                let _ = out.write_fmt(format_args!("`-{}", range_to_inclusive.last));
            }
            Self::Exact(val) => {
                if 0.ne(val) {
                    let _ = out.write_fmt(format_args!("`{val}"));
                }
            }
        }
    }
}

impl const RangeBounds<u32> for GenericCountRequirement {
    fn start_bound(&self) -> Bound<&u32> {
        match self {
            Self::AtLeast(range_from) => range_from.start_bound(),
            Self::NoMoreThan(range_to_inclusive) => range_to_inclusive.start_bound(),
            Self::Exact(v) => Bound::Included(v),
        }
    }
    fn end_bound(&self) -> Bound<&u32> {
        match self {
            Self::AtLeast(range_from) => range_from.end_bound(),
            Self::NoMoreThan(range_to_inclusive) => range_to_inclusive.end_bound(),
            Self::Exact(v) => Bound::Included(v),
        }
    }
}

impl PartialEq<u32> for GenericCountRequirement {
    fn eq(&self, other: &u32) -> bool {
        matches!(self, Self::Exact(x) if x.eq(other))
    }
}
