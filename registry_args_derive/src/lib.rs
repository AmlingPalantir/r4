extern crate proc_macro2;
extern crate proc_macro;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::export::Span;

#[proc_macro_derive(RegistryArgs, attributes(RegistryArgsName))]
pub fn derive_registry_args(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let d = match &ast.data {
        Data::Struct(d) => d,
        _ => panic!("#[derive(RegistryArgs)] on something unexpected"),
    };
    let d = match &d.fields {
        Fields::Named(d) => d,
        _ => panic!("#[derive(RegistryArgs)] on something unexpected"),
    };

    let pairs: Vec<_> = d.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let user_name = compute_user_name(&f.attrs, field_name.to_string());
        return (field_name, user_name);
    }).collect();
    let mut help_meta_suffix = String::new();
    for (_field_name, user_name) in pairs.iter() {
        help_meta_suffix.push_str(&format!(",{}", user_name));
    }
    let help_meta_suffix = LitStr::new(&help_meta_suffix, Span::call_site());
    let argct = pairs.len();
    let ctor_fields: Vec<_> = pairs.iter().enumerate().map(|(i, (field_name, user_name))| {
        let prefix = LitStr::new(&format!("While parsing {}", user_name), Span::call_site());
        let mangle = quote! { .map_err(|e| e.label(#prefix)) };
        return quote! {
            #field_name: ::registry_args::RegistryArg::parse(args[#i]) #mangle ?,
        };
    }).collect();

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics ::registry_args::RegistryArgs for #ident #ty_generics #where_clause {
            fn help_meta_suffix() -> &'static str {
                return #help_meta_suffix;
            }

            fn argct() -> usize {
                return #argct;
            }

            fn parse(args: &[&str]) -> ::validates::ValidationResult<Self> {
                return Result::Ok(#ident {
                    #( #ctor_fields )*
                });
            }
        }
    };

    return TokenStream::from(gen);
}

fn compute_user_name(attrs: &Vec<Attribute>, default_name: String) -> String {
    return attrs.iter().filter_map(|a| {
        let a = a.interpret_meta()?;
        if a.name() != "RegistryArgsName" {
            return None;
        }
        match a {
            Meta::NameValue(ref nv) => {
                match nv.lit {
                    Lit::Str(ref s) => {
                        return Some(s.value());
                    }
                    _ => {
                        panic!("Unexpected RegistryArgsName attribute: {:?}", a);
                    }
                }
            }
            _ => {
                panic!("Unexpected RegistryArgsName attribute: {:?}", a);
            }
        }
    }).next().unwrap_or(default_name);
}
