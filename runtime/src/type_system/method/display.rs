use std::fmt;

use enumflags2::BitFlags;

use super::{Method, Parameter};

pub struct Display<'a, T>(&'a Method<T>, BitFlags<MethodDisplayOptions>);
pub struct ParameterDisplay<'a>(&'a Parameter);

#[repr(u8)]
#[derive(Clone, Copy)]
#[enumflags2::bitflags(default = WithArgs | WithReturn)]
pub enum MethodDisplayOptions {
    WithReturn,
    WithCallConvention,
    WithArgs,
}

impl<T> Method<T> {
    pub fn display<'a>(&'a self, options: BitFlags<MethodDisplayOptions>) -> Display<'a, T> {
        Display(self, options)
    }
}

impl Parameter {
    pub fn display<'a>(&'a self) -> ParameterDisplay<'a> {
        ParameterDisplay(self)
    }
}

impl<T> fmt::Display for Display<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1.contains(MethodDisplayOptions::WithCallConvention) {
            write!(f, r#"extern "{}" "#, self.0.call_convention)?;
        }

        if self.1.contains(MethodDisplayOptions::WithReturn) {
            write!(f, "{:?}", self.0.return_type)?;
        }

        write!(f, "`{}`", self.0.name())?;
        if self.1.contains(MethodDisplayOptions::WithArgs) {
            write!(f, "(")?;
            if let Some(a) = self.0.args().get(0) {
                write!(f, "{}", a.display())?;
            }
            for a in self.0.args().iter().skip(1) {
                write!(f, ", {}", a.display())?;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl fmt::Display for ParameterDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = self.0.attr.impl_flags().iter();
        if let Some(flag) = flags.next() {
            write!(f, "{flag}")?;
        }
        for flag in flags {
            write!(f, " {flag}")?;
        }
        write!(f, " {:?}", self.0.ty)?;

        Ok(())
    }
}
