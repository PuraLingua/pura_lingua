use std::{fmt::Display, marker::Destruct};

use binary_proc_macros::{ReadFromSection, WriteToSection};
use global_proc_macros::{DeriveMap, Transpose, WithType};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{IRegisterAddr, RegisterAddr, ShortRegisterAddr};

#[repr(u8)]
#[derive(Debug, Clone, WithType, ReadFromSection, WriteToSection, Transpose, DeriveMap)]
#[transpose(TTypeRef, TFieldRef)]
#[with_type(derive_const = (Clone, PartialEq, Eq))]
#[with_type(derive = (Copy, IntoPrimitive, TryFromPrimitive, Debug, ReadFromSection, WriteToSection))]
pub enum Instruction_Set<TTypeRef, TFieldRef, TRegisterAddr>
where
    TRegisterAddr: IRegisterAddr,
{
    Common {
        val: TRegisterAddr,
        container: TRegisterAddr,
        field: TFieldRef,
    },
    This {
        val: TRegisterAddr,
        field: TFieldRef,
    },
    Static {
        val: TRegisterAddr,
        ty: TTypeRef,
        field: TFieldRef,
    },
}

impl<TTypeRef, TFieldRef> Instruction_Set<TTypeRef, TFieldRef, RegisterAddr> {
    pub const fn try_into_short(
        self,
    ) -> Result<Instruction_Set<TTypeRef, TFieldRef, ShortRegisterAddr>, Self>
    where
        Self: [const] Destruct,
    {
        match self {
            Instruction_Set::Common {
                val,
                container,
                field,
            } => match val.try_into_short().and_then({
                struct MapContainer {
                    container: RegisterAddr,
                }
                impl const FnOnce<(ShortRegisterAddr,)> for MapContainer {
                    type Output = Option<(ShortRegisterAddr, ShortRegisterAddr)>;

                    #[inline(always)]
                    extern "rust-call" fn call_once(
                        self,
                        (val,): (ShortRegisterAddr,),
                    ) -> Self::Output {
                        let Self { container } = self;
                        struct Zip {
                            val: ShortRegisterAddr,
                        }

                        impl const FnOnce<(ShortRegisterAddr,)> for Zip {
                            type Output = (ShortRegisterAddr, ShortRegisterAddr);

                            #[inline(always)]
                            extern "rust-call" fn call_once(
                                self,
                                (container,): (ShortRegisterAddr,),
                            ) -> Self::Output {
                                let Self { val } = self;
                                (val, container)
                            }
                        }
                        container.try_into_short().map(Zip { val })
                    }
                }
                MapContainer { container }
            }) {
                Some((val, container)) => Ok(Instruction_Set::Common {
                    val,
                    container,
                    field,
                }),
                None => Err(Instruction_Set::Common {
                    val,
                    container,
                    field,
                }),
            },
            Instruction_Set::This { val, field } => match val.try_into_short() {
                Some(val) => Ok(Instruction_Set::This { val, field }),
                None => Err(Instruction_Set::This { val, field }),
            },
            Instruction_Set::Static { val, ty, field } => match val.try_into_short() {
                Some(val) => Ok(Instruction_Set::Static { val, ty, field }),
                None => Err(Instruction_Set::Static { val, ty, field }),
            },
        }
    }
}

impl<TTypeRef, TFieldRef, TRegisterAddr: IRegisterAddr> Display
    for Instruction_Set<TTypeRef, TFieldRef, TRegisterAddr>
where
    TTypeRef: Display,
    TFieldRef: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction_Set::Common {
                val,
                container,
                field,
            } => f.write_fmt(format_args!(" {val:#x} -> {container}.{field}")),
            Instruction_Set::This { val, field } => {
                f.write_fmt(format_args!(" {val:#x} -> this.{field}"))
            }
            Instruction_Set::Static { val, ty, field } => {
                f.write_fmt(format_args!("Static {val:#x} -> {ty}.{field}"))
            }
        }
    }
}
