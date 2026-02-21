/// # Examples
/// ```rust,ignore
/// match_iota! {
///     match 2 => {
///         iota => 1,
///         iota => 2,
///         iota => 3,
///     } with iota type u64;
///     _ => -1
/// }
/// ```
pub macro match_iota(
        match $e:expr => {
            $(
                $i:ident => $x:expr
            ),* $(,)?
        } $t:ty
        $(;_ => $alt:expr)?
    ) {
    $crate::paste::paste! {
        {
            $crate::iota::iota!{
                $(
                    ${ignore($x)}
                    const [<$i:snake:upper _ ${index()}>]: $t = iota;
                )*
            }
            match $e {
                $(
                    [<$i:snake:upper _ ${index()}>] => $x,
                )*
                $(_ => $alt)?
            }
        }
    }
}

/// # Examples
/// ```rust,ignore
/// hash_map! {
///     "a" => "a"
/// }
/// ```
pub macro hash_map($(
        $k:expr => $v:expr
    ),* $(,)?) {{
    let mut map = ::std::collections::HashMap::with_capacity(${count($k)});
    $(
        map.insert($k, $v);
    )*
    map
}}

pub macro string_name($s:literal) {
    $crate::StringName::from_static_str($s)
}

pub macro t_print($($tt:tt)*) {
    print!("[{}:{}:{}] {}", file!(), line!(), column!(), format!($($tt)*))
}

pub macro t_println($($tt:tt)*) {
    println!("[{}:{}:{}] {}", file!(), line!(), column!(), format!($($tt)*))
}

pub macro d_print($($tt:tt)*) {
    #[cfg(debug_assertions)]
    {
        print!($($tt)*)
    }
}

pub macro d_println($($tt:tt)*) {
    #[cfg(debug_assertions)]
    {
        println!($($tt)*)
    }
}

pub macro dt_print($($tt:tt)*) {
    #[cfg(debug_assertions)]
    {
        $crate::macros::t_print!($($tt)*)
    }
}

pub macro dt_println($($tt:tt)*) {
    #[cfg(debug_assertions)]
    {
        $crate::macros::t_println!($($tt)*)
    }
}

pub macro tt_print($($tt:tt)*) {
    #[cfg(test)]
    {
        $crate::macros::t_print!($($tt)*)
    }
}

pub macro tt_println($($tt:tt)*) {
    #[cfg(test)]
    {
        $crate::macros::t_println!($($tt)*)
    }
}

pub macro fatal_ret($msg:literal) {
    return Err($crate::errors::FatalError::new($msg).into());
}

pub macro const_assert($e:expr) {
    const _: () = {
        #[allow(unused)]
        fn f()
        where
            $crate::assertions::ConstAssert<{ $e }>: $crate::assertions::SuccessAssert,
        {
        }
    };
}

/* cSpell:disable-next-line */
/// Copied from [stdext-rs](https://github.com/popzxc/stdext-rs/blob/dc03b4afa28b3a1d2451ca54ad252244f029099b/src/macros.rs#L63)
pub macro function_name() {{
    // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
    fn f() {}
    fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }
    let name = type_name_of(f);
    // `3` is the length of the `::f`.
    &name[..name.len() - 3]
}}

pub macro warn_if_default_impl() {
    $crate::macros::dt_println!("DEFAULT method {}", $crate::macros::function_name!())
}

pub macro make_vtable(
@VTABLE: $t_vtable:ident;
@ORIGIN: $t_origin:ty;
$i_ident:ident + $m_ident:ident;
    $(
        $name:ident: $pre_op:ident ($($arg:ident),* $(,)?)
    ),* $(,)?
@rest
    $($tt:tt)*
) {{
    #[inline(always)]
    #[allow(unused)]
    #[track_caller]
    fn $i_ident<'a>(pointer: *const $t_origin) -> &'a $t_origin {
        unsafe { pointer.as_ref_unchecked() }
    }
    #[inline(always)]
    #[allow(unused)]
    #[track_caller]
    fn $m_ident<'a>(pointer: *mut $t_origin) -> &'a mut $t_origin {
        unsafe { pointer.as_mut_unchecked() }
    }
    $t_vtable {$(
        #[allow(unused_unsafe)]
        $name: |this $(,$arg)*| unsafe {
            $pre_op(this.cast::<$t_origin>()).$name($($arg),*)
        },
    )* $($tt)*}
}}

#[cfg(test)]
#[allow(unused)]
mod tests4proc_macro {
    use proc_macros::UnwrapEnum;

    use super::*;

    #[derive(UnwrapEnum)]
    #[unwrap_enum(ref, ref_mut)]
    enum TestUnwrap {
        A,
        B,
        C,
        #[unwrap_enum(owned)]
        D(u64),
        E(u8),
        Multi(u8, u64),
    }
}
