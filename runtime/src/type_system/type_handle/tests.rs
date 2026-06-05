use std::{num::NonZero, ptr::NonNull};

use crate::type_system::{
    class::Class,
    interface::Interface,
    r#struct::Struct,
    type_handle::{FlattenedNonGenericTypeHandle, FlattenedTypeHandle},
};

fn get_aligned_random_ptr<T>() -> NonNull<T> {
    let random_addr = rand::random_range(align_of::<T>()..(usize::MAX & !align_of::<T>()));

    let mask = align_of::<T>() - 1;
    let offset = (align_of::<T>() - (random_addr & mask)) & mask;

    NonNull::without_provenance(unsafe { NonZero::new_unchecked(random_addr.wrapping_add(offset)) })
}

#[test]
fn test_non_generic_flatten() {
    let classes = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Class>())
        .chain(std::iter::once(NonNull::<Class>::dangling()))
        .collect::<Vec<_>>();
    let structs = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Struct>())
        .chain(std::iter::once(NonNull::<Struct>::dangling()))
        .collect::<Vec<_>>();
    let interfaces = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Interface>())
        .chain(std::iter::once(NonNull::<Interface>::dangling()))
        .collect::<Vec<_>>();

    for ptr in classes {
        let flatten = FlattenedNonGenericTypeHandle::Class(ptr);
        assert_eq!(flatten.try_into_class(), Some(ptr));
    }
    for ptr in structs {
        let flatten = FlattenedNonGenericTypeHandle::Struct(ptr);
        assert_eq!(flatten.try_into_struct(), Some(ptr));
    }
    for ptr in interfaces {
        let flatten = FlattenedNonGenericTypeHandle::Interface(ptr);
        assert_eq!(flatten.try_into_interface(), Some(ptr));
    }
}

#[test]
fn test_flatten() {
    let classes = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Class>())
        .chain(std::iter::once(NonNull::<Class>::dangling()))
        .collect::<Vec<_>>();
    let structs = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Struct>())
        .chain(std::iter::once(NonNull::<Struct>::dangling()))
        .collect::<Vec<_>>();
    let interfaces = (0..100)
        // cSpell:disable-next-line
        .map(|_| get_aligned_random_ptr::<Interface>())
        .chain(std::iter::once(NonNull::<Interface>::dangling()))
        .collect::<Vec<_>>();

    let method_generics = (0..100)
        // cSpell:disable-next-line
        .map(|_| rand::TryRng::try_next_u32(&mut rand::rngs::SysRng).unwrap())
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let type_generics = (0..100)
        // cSpell:disable-next-line
        .map(|_| rand::TryRng::try_next_u32(&mut rand::rngs::SysRng).unwrap())
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    for ptr in classes {
        let flatten = FlattenedTypeHandle::Class(ptr);
        assert_eq!(flatten.try_into_class(), Some(ptr));
    }
    for ptr in structs {
        let flatten = FlattenedTypeHandle::Struct(ptr);
        assert_eq!(flatten.try_into_struct(), Some(ptr));
    }
    for ptr in interfaces {
        let flatten = FlattenedTypeHandle::Interface(ptr);
        assert_eq!(flatten.try_into_interface(), Some(ptr));
    }

    for i in method_generics {
        let flatten = FlattenedTypeHandle::MethodGeneric(i);
        assert_eq!(flatten.try_into_method_generic(), Some(i));
    }

    for i in type_generics {
        let flatten = FlattenedTypeHandle::TypeGeneric(i);
        assert_eq!(flatten.try_into_type_generic(), Some(i));
    }
}
