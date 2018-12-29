extern crate proc_macro;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
use syn::export::Span;

#[proc_macro_derive(Validates)]
pub fn derive_validates(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ctor_args;
    let struct_args;
    let clone_args;
    match &ast.data {
        Data::Struct(d) => match &d.fields {
            Fields::Named(d) => {
                let ctor_fields: Vec<_> = d.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    return quote! {
                        #name: ::validates::Validates::validate(self.#name),
                    };
                }).collect();
                ctor_args = quote! { { #( #ctor_fields )* } };
                let struct_fields: Vec<_> = d.named.iter().map(|f| {
                    let vis = &f.vis;
                    let name = f.ident.as_ref().unwrap();
                    let ty = &f.ty;
                    return quote! {
                        #vis #name: <#ty as ::validates::Validates>::Target,
                    };
                }).collect();
                struct_args = quote! { { #( #struct_fields )* } };
                let clone_fields: Vec<_> = d.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    return quote! {
                        #name: ::std::clone::Clone::clone(&self.#name),
                    };
                }).collect();
                clone_args = quote! { { #( #clone_fields )* } };
            },
            Fields::Unnamed(d) => {
                let ctor_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(name, _f)| {
                    return quote! {
                        ::validates::Validates::validate(self.#name),
                    };
                }).collect();
                ctor_args = quote! { ( #( #ctor_fields )* ) };
                let struct_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(_name, f)| {
                    let vis = &f.vis;
                    let ty = &f.ty;
                    return quote! {
                        #vis <#ty as ::validates::Validates>::Target,
                    };
                }).collect();
                struct_args = quote! { ( #( #struct_fields )* ); };
                let clone_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(name, _f)| {
                    return quote! {
                        ::std::clone::Clone::clone(&self.#name),
                    };
                }).collect();
                clone_args = quote! { ( #( #clone_fields )* ) };
            },
            Fields::Unit => {
                ctor_args = quote! { () };
                struct_args = quote! { () };
                clone_args = quote! { () };
            },
        },
        _ => panic!(),
    };

    let vis = &ast.vis;
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident_validated = Ident::new(&format!("{}Validated", ident), Span::call_site());
    let gen = quote! {
        impl #impl_generics Validates for #ident #ty_generics #where_clause {
            type Target = #ident_validated #ty_generics;

            fn validate(self) -> Self::Target {
                return #ident_validated #ctor_args;
            }
        }

        #vis struct #ident_validated #impl_generics #struct_args #where_clause

        impl #impl_generics Clone for #ident_validated #ty_generics #where_clause {
            fn clone(&self) -> Self {
                return #ident_validated #clone_args;
            }
        }
    };

    return TokenStream::from(gen);
}
