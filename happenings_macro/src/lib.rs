extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

#[proc_macro_attribute]
pub fn generate_new(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let new_struct_name = syn::Ident::new(&format!("New{}", struct_name), struct_name.span());
    let db_struct_name = syn::Ident::new(&format!("Db{}", struct_name), struct_name.span());

    let to_method_name = format_ident!("to_{}", struct_name.to_string().to_lowercase());

    let attrs = input.attrs.clone();

    let new_fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .filter(|f| f.ident.as_ref().unwrap() != "id") // Exclude 'id' field
                    .collect::<Vec<_>>(),
                _ => unimplemented!(), // Add support for other types (unnamed fields, tuples) as needed
            }
        }
        _ => unimplemented!(), // Support only struct, not enums or unions
    };

    let db_fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .map(|f| {
                        if f.ident.as_ref().unwrap().to_string().ends_with("id") {
                            syn::Field {
                                ty: syn::parse_str::<syn::Type>("surrealdb::sql::Thing").unwrap(),
                                ..f.clone()
                            }
                        } else {
                            f.clone()
                        }
                    }) // Turn anything that ends in _id to a thing
                    .collect::<Vec<_>>(),
                _ => unimplemented!(), // Add support for other types (unnamed fields, tuples) as needed
            }
        }
        _ => unimplemented!(), // Support only struct, not enums or unions
    };

    // Extract field names
    let field_names =
        if let Data::Struct(data_struct) = &input.data {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().clone())
                    .collect::<Vec<_>>(),
                _ => unimplemented!(),
            }
        } else {
            unimplemented!()
        };

    let field_assignments: Vec<_> = new_fields
        .iter()
        .filter(|field| field.ident.as_ref().map_or(false, |name| name != "id"))
        .map(|field| {
            let field_name = &field.ident;
            quote! { #field_name: self.#field_name }
        })
        .collect();

    let db_field_assignments: Vec<_> =
        new_fields
            .iter()
            .filter(|field| field.ident.as_ref().map_or(false, |name| name != "id"))
            .map(|field| {
                let field_name = &field.ident;
                if field_name.as_ref().unwrap().to_string().ends_with("_id") {
                    quote! { #field_name: item.#field_name.to_string() }
                } else {
                    quote! { #field_name: item.#field_name }
                }
            })
            .collect();

    let new_struct = quote! {
                #(#attrs)*
                pub struct #new_struct_name {
                    #(#new_fields),*
                }
    };

    let db_struct =
        quote! {
                #(#attrs)*
                pub struct #db_struct_name {
                    #(#db_fields),*
                }
        };

    let impl_new_struct = quote! {
        impl  #new_struct_name  {
            pub fn #to_method_name(self, id: String) -> #struct_name {
                #struct_name {
                     id,
                     #(#field_assignments),*
                    }
                }
            }
    };

    let impl_db_struct = quote! {
        impl From<#db_struct_name> for #struct_name {
            fn from(item: #db_struct_name) -> Self {
                Self {
                    id: item.id.to_string(),
                    #(#db_field_assignments),*
                    // #(
                    //     #db_field_assignments
                    //     // #field_names: item.#field_names.into(),
                    // )*,
                }
            }
        }
    };
    let gen =
        quote! {
            #input
            #new_struct
            #impl_new_struct
            #db_struct
            #impl_db_struct
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

