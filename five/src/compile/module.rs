use crate::analysis::{
    FunctionDescription, GenericsInfo, ModuleInfo, ParameterInfo, TypeDescription,
};

use super::{context::CompiledContext, Compiled, Compiler};

#[derive(Clone)]
pub struct CompiledModule {
    pub module_name: syn::Ident,
    pub context: CompiledContext,
    pub others: Vec<syn::Item>,
}

impl Compiled<ModuleInfo> for CompiledModule {
    fn emit(&self) -> proc_macro2::TokenStream {
        use quote::quote;

        let module_name = &self.module_name;
        let mut context = self.context.clone();

        // Create PascalCase trait name from module name
        let module_str = module_name.to_string();
        
        // Convert snake_case to PascalCase
        let name = module_str.split('_')
            .map(|part| {
                if part.is_empty() {
                    String::new()
                } else {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                }
            })
            .collect::<String>();
            
        let trait_name = syn::Ident::new(
            &name,
            module_name.span()
        );

        // Update the trait name to match the module name in PascalCase
        context.context_trait.ident = trait_name.clone();

        // Get generics from the context base struct
        let (_impl_generics, ty_generics, _where_clausee) = context.base.generics.split_for_impl();

        // Update the impl blocks to implement the renamed trait with generics
        for impl_block in &mut context.context_methods {
            impl_block.implemented_traits = vec![syn::parse_quote!(#trait_name #ty_generics)];
            impl_block.self_ty = syn::parse_quote!(Context #ty_generics);
            impl_block.generics = GenericsInfo::from_syn_generics(&context.base.generics);
            impl_block.attrs = context.attrs.clone();
        }
        

        // Get the struct fields from the context base
        let field_names = match &context.base.fields {
            syn::Fields::Named(fields) => fields.named.iter().map(|f| &f.ident),
            _ => panic!("Expected named fields"),
        };
        let field_types = match &context.base.fields {
            syn::Fields::Named(fields) => fields.named.iter().map(|f| &f.ty),
            _ => panic!("Expected named fields"),
        };

        let field_names = field_names.collect::<Vec<_>>();
        let field_types = field_types.collect::<Vec<_>>();

        let bind_fn_name = syn::Ident::new("bind", proc_macro2::Span::call_site());

        let context_type = syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: {
                    let mut segments = syn::punctuated::Punctuated::new();
                    segments.push(syn::PathSegment {
                        ident: syn::Ident::new("Context", proc_macro2::Span::call_site()),
                        arguments: syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: syn::Token![<](proc_macro2::Span::call_site()),
                                args: context
                                    .base
                                    .generics
                                    .params
                                    .iter()
                                    .map(|param| match param {
                                        syn::GenericParam::Type(t) => syn::GenericArgument::Type(
                                            syn::Type::Path(syn::TypePath {
                                                qself: None,
                                                path: syn::Path::from(t.ident.clone()),
                                            }),
                                        ),
                                        syn::GenericParam::Lifetime(l) => {
                                            syn::GenericArgument::Lifetime(l.lifetime.clone())
                                        }
                                        syn::GenericParam::Const(c) => syn::GenericArgument::Const(
                                            syn::Expr::Path(syn::ExprPath {
                                                attrs: vec![],
                                                qself: None,
                                                path: syn::Path::from(c.ident.clone()),
                                            }),
                                        ),
                                    })
                                    .collect(),
                                gt_token: syn::Token![>](proc_macro2::Span::call_site()),
                            },
                        ),
                    });
                    segments
                },
            },
        });

        let bind_fn_body = syn::Block {
            brace_token: syn::token::Brace::default(),
            stmts: vec![syn::Stmt::Expr(
                syn::Expr::Struct(syn::ExprStruct {
                    attrs: vec![],
                    qself: None,
                    path: match &context_type {
                        syn::Type::Path(type_path) => type_path.path.clone(),
                        _ => panic!("Expected Type::Path"),
                    },
                    brace_token: syn::token::Brace::default(),
                    fields: {
                        let mut fields = syn::punctuated::Punctuated::new();
                        
                        // Add all non-phantom fields from parameters
                        for field_name in field_names.iter() {
                            let field_name_str = field_name.as_ref().unwrap().to_string();
                            if !field_name_str.starts_with("_phantom_") {
                                fields.push(syn::FieldValue {
                                    attrs: vec![],
                                    member: syn::Member::Named(field_name.as_ref().unwrap().clone()),
                                    colon_token: Some(syn::Token![:](proc_macro2::Span::call_site())),
                                    expr: syn::Expr::Path(syn::ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: syn::Path::from(field_name.as_ref().unwrap().clone()),
                                    }),
                                });
                            }
                        }
                        
                        // Add default PhantomData for phantom fields
                        for field_name in field_names.iter() {
                            let field_name_str = field_name.as_ref().unwrap().to_string();
                            if field_name_str.starts_with("_phantom_") {
                                fields.push(syn::FieldValue {
                                    attrs: vec![],
                                    member: syn::Member::Named(field_name.as_ref().unwrap().clone()),
                                    colon_token: Some(syn::Token![:](proc_macro2::Span::call_site())),
                                    expr: syn::parse_quote!(::std::marker::PhantomData),
                                });
                            }
                        }
                        
                        fields
                    },
                    dot2_token: None,
                    rest: None,
                }),
                None,
            )],
        };

        // Create appropriate return type with all generic parameters preserved
        let return_type = {
            let (_, type_generics, _) = context.base.generics.split_for_impl();
            
            syn::parse_quote!(-> impl #trait_name #type_generics)
        };

        let params: Vec<ParameterInfo> = field_names
            .iter()
            .zip(field_types.iter())
            .filter(|(name, _)| {
                // Filter out self and _phantom_ parameters
                let name_str = name.as_ref().unwrap().to_string();
                *name.as_ref().unwrap() != "self" && !name_str.starts_with("_phantom_")
            })
            .map(|(name, ty)| {
                let param = ParameterInfo::Typed {
                    name: name.as_ref().unwrap().clone(),
                    ty: (*ty).clone(),
                };

                param
            })
            .collect();

        let bind_fn = FunctionDescription::new_implementation(
            bind_fn_name,
            params,
            GenericsInfo::from_syn_generics(&context.base.generics),
            return_type,
            bind_fn_body,
            None,
            vec![],
        );

        let bind_fn = bind_fn.compile();

        let bind_fn = bind_fn.emit();
        let context = context.emit();
        let others = &self.others;

        
        quote! {
            #context
            #bind_fn
            #(#others)*
        }
    }
}

impl Compiler<ModuleInfo> for ModuleInfo {
    fn compile(&self) -> CompiledModule {
        // Compile the context first
        let compiled_context = self.context.compile();

        CompiledModule {
            module_name: self.module_name.clone(),
            context: compiled_context,
            others: self
                .others
                .iter()
                .filter_map(|item| match item {
                    TypeDescription::Other(item) => Some(item.clone()),
                    _ => None,
                })
                .collect(), // Just clone the others without compilation
        }
    }

    type Output = CompiledModule;
}
