pub macro define_derive_macros($(
    $Name:ident [$($attr:ident),* $(,)?] => $_impl:path;
)*) {$(
    #[allow(nonstandard_style)]
    #[proc_macro_derive($Name, attributes($($attr),*))]
    pub fn ${concat(derive_, $Name)}(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
        let input = ::syn::parse_macro_input!(input as ::syn::DeriveInput);
        ($_impl)(input)
            .unwrap_or_else(|err| err.into_compile_error())
            .into()
    }
)*}

pub macro define_macros($(
    $name:ident => $_impl:path as $Type:ty;
)*) {$(
    #[proc_macro]
    pub fn $name(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
        let input = ::syn::parse_macro_input!(input as $Type);
        ($_impl)(input)
            .unwrap_or_else(|err| err.into_compile_error())
            .into()
    }
)*}
