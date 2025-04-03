use impl_block::CompiledImplBlock;
use proc_macro2::TokenStream;
use quote::ToTokens;
use role::{to_role_name, CompiledRole};
use std::collections::HashMap;
use syn::{visit_mut::VisitMut, Block, Expr, ItemStruct, Member};

use crate::analysis::{ContextInfo, FunctionDescription, GenericsInfo, ImplBlockInfo, TraitInfo};

use super::*;

impl Compiler<ContextInfo> for ContextInfo {
    fn compile(&self) -> CompiledContext {
        // Create the context trait from impl block methods
        let blocks = self
            .impl_blocks
            .iter()
            .map(|block: &ImplBlockInfo| {
                let mut block = block.clone();
                block.attrs = self.attrs.clone();
                block
            })
            .collect::<Vec<ImplBlockInfo>>();
        let trait_methods =
            blocks
                .iter()
                .flat_map(|block| {
                    block.functions.iter().map(|f| {
                        match f {
                            FunctionDescription::Implementation { name, params, generics, output, asyncness, .. } => {
                                let param_tokens = params.iter().map(|p| p.to_token_stream());

                                let generic_params = generics.get_params();
                                let where_clause = generics.get_where_clause();

                                // Only add angle brackets if we have generic parameters
                                let generic_tokens = if !generic_params.is_empty() {
                                    quote::quote!(<#(#generic_params),*>)
                                } else {
                                    quote::quote!()
                                };

                                // Handle async methods
                                let method: syn::TraitItem = if asyncness.is_some() {
                                    syn::parse_quote! {
                                        async fn #name #generic_tokens (#(#param_tokens),*) #output #where_clause;
                                    }
                                } else {
                                    syn::parse_quote! {
                                        fn #name #generic_tokens (#(#param_tokens),*) #output #where_clause;
                                    }
                                };

                                // Trait methods don't have visibility modifiers, so we can just return the method
                                method
                            },
                            _ => panic!("Expected implementation")
                        }
                    })
                })
                .collect::<Vec<syn::TraitItem>>();

        let base = self.compile_struct();

        // Get the type name and generics from the base struct
        let type_name = &base.ident;

        let generics = GenericsInfo::from_syn_generics(&base.generics);

        // Create roles_map for method compilation
        let roles_map: HashMap<String, TraitInfo> = self
            .roles
            .iter()
            .map(|r| (to_role_name(&r.name.to_string()), r.contract.clone()))
            .collect();

        let trait_name = syn::Ident::new("Context", proc_macro2::Span::call_site());

        // Create the trait definition with async_trait if needed
        let trait_def = quote::quote! {
            pub trait #trait_name {
                #(#trait_methods)*
            }
        };

        let context_trait = syn::parse2(trait_def).unwrap();

        // Compile roles
        let roles = self
            .roles
            .iter()
            .map(|r| {
                let mut compiled_role = r.compile(&roles_map);
                compiled_role.impl_block.self_ty = syn::parse_quote!(#type_name #generics);
                compiled_role.impl_block.generics = generics.clone();
                compiled_role
            })
            .collect();

        // Create impl blocks that implement Context
        let context_methods = blocks
            .iter()
            .map(|b| {
                let mut impl_block = self.compile_context_methods(&roles_map, b.clone());
                impl_block.generics = generics.clone();
                impl_block.implemented_traits = vec![syn::parse_quote!(Context)];
                impl_block
            })
            .collect();

        CompiledContext {
            roles,
            context_methods,
            base,
            context_trait,
            attrs: self.attrs.clone(),
        }
    }

    type Output = CompiledContext;
}

#[derive(Clone)]
pub struct CompiledContext {
    pub roles: Vec<CompiledRole>,
    pub context_methods: Vec<CompiledImplBlock>,
    pub base: ItemStruct,
    pub context_trait: syn::ItemTrait,
    pub attrs: Vec<syn::Attribute>,
}

impl Compiled<ContextInfo> for CompiledContext {
    fn emit(&self) -> TokenStream {
        let mut ts = TokenStream::new();

        // First emit the struct definition
        ts.extend(self.base.to_token_stream());

        // Then emit the trait definition
        ts.extend(self.context_trait.to_token_stream());

        // Then emit the role implementations
        ts.extend(TokenStream::from_iter(self.roles.iter().map(|r| r.emit())));

        // Finally emit the context method implementations
        ts.extend(TokenStream::from_iter(
            self.context_methods.iter().flat_map(|r| r.emit()),
        ));

        ts
    }
}

impl ContextInfo {
    fn is_primitive_type(ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    matches!(
                        segment.ident.to_string().as_str(),
                        "i32" | "i64" | "f32" | "f64" | "bool" | "char" | "str"
                    )
                } else {
                    false
                }
            }
            _ => false, // Treat all other types as non-primitive
        }
    }

    fn rewrite_role_access(&self, roles: &HashMap<String, TraitInfo>, block: &mut Block) {
        struct RoleMethodRewriter<'a> {
            roles: &'a HashMap<String, TraitInfo>,
        }

        impl VisitMut for RoleMethodRewriter<'_> {
            fn visit_expr_mut(&mut self, expr: &mut Expr) {
                if let Expr::MethodCall(method_call) = expr {
                    if let Expr::Field(field_expr) = &*method_call.receiver {
                        if let Expr::Path(base_path) = &*field_expr.base {
                            if let Some(ident) = base_path.path.get_ident() {
                                if ident == "self" {
                                    if let Member::Named(role_name) = &field_expr.member {
                                        if let Some(role_trait) =
                                            self.roles.get(&role_name.to_string())
                                        {
                                            // Only rewrite if method is in role but not in contract
                                            let method_name = method_call.method.to_string();
                                            let is_role_method = role_trait
                                                .functions
                                                .iter()
                                                .any(|m| *m.get_name() == method_name);

                                            if !is_role_method {
                                                // Create the new method name: role_method
                                                let new_method_name = syn::Ident::new(
                                                    &format!(
                                                        "{}_{}",
                                                        role_name, method_call.method
                                                    ),
                                                    method_call.method.span(),
                                                );

                                                // Create new method call with the same arguments
                                                let new_expr =
                                                    Expr::MethodCall(syn::ExprMethodCall {
                                                        attrs: method_call.attrs.clone(),
                                                        receiver: Box::new(Expr::Path(
                                                            syn::ExprPath {
                                                                attrs: vec![],
                                                                qself: None,
                                                                path: syn::parse_quote!(self),
                                                            },
                                                        )),
                                                        dot_token: method_call.dot_token,
                                                        method: new_method_name,
                                                        turbofish: None,
                                                        paren_token: method_call.paren_token,
                                                        args: method_call.args.clone(),
                                                    });

                                                *expr = new_expr;
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }

        let mut rewriter = RoleMethodRewriter { roles };
        rewriter.visit_block_mut(block);
    }

    fn compile_context_methods(
        &self,
        roles_map: &HashMap<String, TraitInfo>,
        impl_block: ImplBlockInfo,
    ) -> CompiledImplBlock {
        let functions = impl_block
            .functions
            .iter()
            .map(|func| {
                match func {
                    FunctionDescription::Implementation {
                        name,
                        params,
                        generics,
                        output,
                        body,
                        asyncness,
                        attrs,
                    } => {
                        let mut body = body.clone();
                        self.rewrite_role_access(roles_map, &mut body);
                        // Preserve the original parameters including self receiver
                        FunctionDescription::new_implementation(
                            name.clone(),
                            params.clone(), // Keep original params which include self receiver
                            generics.clone(),
                            output.clone(),
                            body,
                            *asyncness,
                            attrs.clone(),
                        )
                    }
                    f => f.clone(),
                }
            })
            .collect();

        ImplBlockInfo {
            functions,
            ..impl_block
        }
        .compile()
    }

    fn compile_struct(&self) -> ItemStruct {
        let mut generics_params = self.generics.get_params().clone();

        // Map properties to their corresponding generic parameters or original types
        let property_generics: Vec<_> = self
            .properties
            .iter()
            .map(|prop| {
                if Self::is_primitive_type(&prop.get_ty()) {
                    // Primitive type: use original type
                    (prop.get_name().clone(), prop.get_ty().clone(), None)
                } else {
                    // Non-primitive type: use a generic parameter with a contract
                    let contract_name = format!(
                        "{}Contract",
                        to_upper_camel_case(&prop.get_name().to_string())
                    );
                    let generic_name = syn::Ident::new(
                        &format!("T{}", to_upper_camel_case(&prop.get_name().to_string())),
                        proc_macro2::Span::call_site(),
                    );
                    let contract_ident =
                        syn::Ident::new(&contract_name, proc_macro2::Span::call_site());

                    // Add the generic parameter to generics
                    generics_params.push(syn::GenericParam::Type(syn::TypeParam {
                        attrs: vec![],
                        ident: generic_name.clone(),
                        bounds: syn::punctuated::Punctuated::from_iter(vec![
                            syn::TypeParamBound::Trait(syn::TraitBound {
                                paren_token: None,
                                modifier: syn::TraitBoundModifier::None,
                                lifetimes: None,
                                path: syn::Path::from(contract_ident.clone()),
                            }),
                        ]),
                        eq_token: None,
                        default: None,
                        colon_token: Some(Default::default()),
                    }));

                    // Return the generic type instead of the original type
                    (
                        prop.get_name().clone(),
                        syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path::from(generic_name),
                        }),
                        Some(contract_ident),
                    )
                }
            })
            .collect();

        // Generate fields for the struct
        let fields: Vec<syn::Field> = property_generics
            .into_iter()
            .map(|(field_name, field_type, _)| syn::Field {
                mutability: syn::FieldMutability::None,
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                ident: Some(field_name),
                colon_token: Some(Default::default()),
                ty: field_type,
            })
            .collect();

        // Finalize generics with additional parameters
        let mut gs = self.generics.to_syn_generics();
        gs.params.extend(generics_params);

        // Construct the struct
        syn::ItemStruct {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            struct_token: syn::token::Struct {
                span: proc_macro2::Span::call_site(),
            },
            ident: self.name.clone(),
            generics: syn::Generics {
                lt_token: Some(Default::default()),
                params: gs.params,
                gt_token: Some(Default::default()),
                where_clause: self.generics.get_where_clause(),
            },
            fields: syn::Fields::Named(syn::FieldsNamed {
                brace_token: Default::default(),
                named: syn::punctuated::Punctuated::from_iter(fields),
            }),
            semi_token: None,
        }
    }
}

fn to_upper_camel_case(input: &str) -> String {
    input
        .split('_') // Split on underscores
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

impl GenericsInfo {
    pub fn to_syn_generics(&self) -> syn::Generics {
        syn::Generics {
            lt_token: if self.get_params().is_empty() {
                None
            } else {
                Some(Default::default())
            },
            params: self.get_params().clone().into_iter().collect(),
            gt_token: if self.get_params().is_empty() {
                None
            } else {
                Some(Default::default())
            },
            where_clause: self.get_where_clause().clone(),
        }
    }
}
