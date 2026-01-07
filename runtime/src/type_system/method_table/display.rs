use std::fmt;

use enumflags2::make_bitflags;

use super::super::method::MethodDisplayOptions;
use super::MethodTable;

pub struct Display<'a, T>(&'a MethodTable<T>);

impl<T> MethodTable<T> {
    pub fn display<'a>(&'a self) -> Display<'a, T> {
        Display(self)
    }
}

impl<T> fmt::Display for Display<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let methods = self.0.methods.read().unwrap();
        for m in &*methods {
            writeln!(f, "{}", unsafe { m.as_ref() }.display(make_bitflags!(MethodDisplayOptions::{WithArgs | WithCallConvention | WithReturn})))?;
        }

        Ok(())
    }
}
