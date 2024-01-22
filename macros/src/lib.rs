extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

#[proc_macro_attribute]
pub fn generate_new(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let new_struct_name = syn::Ident::new(&format!("New{}", struct_name), struct_name.span());
    // let to_method_name = format_ident!("to_{}", struct_name.to_string().to_lowercase());

    let attrs = input.attrs.clone();

    // Remove our 'not_in_new' attribute
    let fields_without_attr = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .map(|f| {
                    // Create a field without the #[not_in_new] attribute
                    let mut field = f.clone();
                    field.attrs = field
                        .attrs
                        .into_iter()
                        .filter(|attr| !attr.path.is_ident("not_in_new"))
                        .collect();
                    field
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let new_fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .filter(|f| f.ident.as_ref().unwrap() != "id" && !has_not_in_new_attr(f)) // Exclude 'id' field
                    .collect::<Vec<_>>(),
                _ => unimplemented!(), // Add support for other types (unnamed fields, tuples) as needed
            }
        }
        _ => unimplemented!(), // Support only struct, not enums or unions
    };

    // let field_assignments: Vec<_> = new_fields
    //     .iter()
    //     // .filter(|field| field.ident.as_ref().map_or(false, |name| name != "id"))
    //     .map(|field| {
    //         let field_name = &field.ident;
    //         quote! { #field_name: self.#field_name }
    //     })
    //     .collect();

    let orig_struct = quote! {
            #(#attrs)*
            pub struct #struct_name {
                #(#fields_without_attr),*
            }
    };

    let new_struct = quote! {
                #(#attrs)*
                pub struct #new_struct_name {
                    #(#new_fields),*
                }
    };

    // let impl_new_struct = quote! {
    //     impl  #new_struct_name  {
    //         pub fn #to_method_name(self, id: String) -> #struct_name {
    //             #struct_name {
    //                  id,
    //                  #(#field_assignments),*
    //                 }
    //             }
    //         }
    // };

    let gen = quote! {
        #orig_struct
        #new_struct
        // #impl_new_struct
    };

    gen.into()
}

fn has_not_in_new_attr(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("not_in_new"))
}

#[proc_macro_attribute]
pub fn generate_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let db_struct_name = syn::Ident::new(&format!("Db{}", struct_name), struct_name.span());

    let attrs = input.attrs.clone();

    let ident_is_thing = |i: &syn::Ident| i.to_string().ends_with("_id") || i == "id";

    // Turn anything that ends in _id to a thing
    let db_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .map(|f| {
                    if ident_is_thing(f.ident.as_ref().unwrap()) {
                        syn::Field {
                            ty: syn::parse_str::<syn::Type>("surrealdb::sql::Thing").unwrap(),
                            ..f.clone()
                        }
                    } else {
                        f.clone()
                    }
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let db_field_assignments: Vec<_> = db_fields
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            if ident_is_thing(field_name.as_ref().unwrap()) {
                quote! { #field_name: item.#field_name.to_string() }
            } else {
                quote! { #field_name: item.#field_name }
            }
        })
        .collect();

    let db_struct = quote! {
            #(#attrs)*
            pub struct #db_struct_name {
                #(#db_fields),*
            }
    };

    let impl_db_struct = quote! {
        impl From<#db_struct_name> for #struct_name {
            fn from(item: #db_struct_name) -> Self {
                Self {
                    #(#db_field_assignments),*
                }
            }
        }
    };
    let gen = quote! {
        #input

        #[cfg(not(target_arch = "wasm32"))]
        #db_struct

        #[cfg(not(target_arch = "wasm32"))]
        #impl_db_struct
    };

    gen.into()
}

fn struct_fields_without_attr(input: &syn::DeriveInput) -> Vec<syn::Field> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .map(|f| {
                    // Create a field without the #[not_in_new] attribute
                    let mut field = f.clone();
                    field.attrs.retain(|attr| !attr.path.is_ident("not_in_new"));
                    field
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn field_has_attr(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("not_in_new"))
}

#[proc_macro_attribute]
pub fn generate_new_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let db_struct_name = syn::Ident::new(&format!("NewDb{}", struct_name), struct_name.span());

    let attrs = input.attrs.clone();

    // Remove our 'not_in_new' attribute
    let fields_without_attr = struct_fields_without_attr(&input);
    let orig_struct = quote! {
            #(#attrs)*
            pub struct #struct_name {
                #(#fields_without_attr),*
            }
    };

    let ident_is_thing = |i: &syn::Ident| i.to_string().ends_with("_id") || i == "id";

    // Turn anything that ends in _id to a thing
    let db_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .filter(|f| f.ident.as_ref().unwrap() != "id") // Exclude 'id' field
                .filter(|f| !field_has_attr(f)) // Exclude fields with #[not_in_new] attribute
                .map(|f| {
                    if ident_is_thing(f.ident.as_ref().unwrap()) {
                        syn::Field {
                            ty: syn::parse_str::<syn::Type>("surrealdb::sql::Thing").unwrap(),
                            ..f.clone()
                        }
                    } else {
                        f.clone()
                    }
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let db_struct = quote! {
            #(#attrs)*
            pub struct #db_struct_name {
                #(#db_fields),*
            }
    };

    let gen = quote! {
        #orig_struct

        #[cfg(not(target_arch = "wasm32"))]
        #db_struct
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn serverfn_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let output = quote! {
        #[tokio::test]
        #fn_sig  {
            let inner = async {#fn_block};

            let runtime = leptos::create_runtime();
            let guard = scopeguard::guard(runtime, |r| r.dispose());

            let db = surrealdb::engine::any::connect("mem://").await.unwrap();
            db.use_ns("test").use_db("test").await.unwrap();

            let app_state = AppState {
                db,
                config: crate::Config::default(),
            };
            leptos::provide_context(app_state);

            inner.await
        }
    };

    output.into()
}

