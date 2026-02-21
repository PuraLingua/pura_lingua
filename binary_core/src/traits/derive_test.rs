#![allow(dead_code)]

use proc_macros::{ReadFromSection, WriteToSection};

#[derive(ReadFromSection, WriteToSection)]
struct Test1 {
    f: u8,
    f2: u16,
}
