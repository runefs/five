use std::collections::HashMap;
use quote::ToTokens;
use impl_block::CompiledImplBlock;
use proc_macro2::TokenStream;
use role::{to_role_name, CompiledRole};
use syn::{visit_mut::VisitMut, Block, Expr, ItemStruct, Member};

use crate::analysis::{ContextInfo, FunctionDescription, GenericsInfo, ImplBlockInfo, TraitInfo};

use super::*;

impl Compiler<ContextInfo> for ContextInfo {
    fn compile(&self) -> CompiledContext {

        // Create the context trait from impl block methods
        let blocks = self.impl_blocks
            .iter()
            .map(|block: &ImplBlockInfo| {
                let mut block = block.clone();
                block.attrs = self.attrs.clone();
                block
            }).collect::<Vec<ImplBlockInfo>>();
        
        // Get all method signatures for the trait, preserving generics
        let trait_methods =
            blocks
                .iter()
                .flat_map(|block| {
                    block.functions.iter().map(|f| {
                        match f {
                            FunctionDescription::Implementation { name, params, generics, output, asyncness, .. } => {
                                let param_tokens = params.iter().map(|p| p.to_token_stream());

                                // Combine generics from the impl block with the struct generics
                                let combined_generics = generics.clone();
                                
                                // Only add angle brackets if we have generic parameters
                                let generic_params = combined_generics.get_params();
                                let where_clause = combined_generics.get_where_clause();
                                
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

        // Extract all generic parameters from the struct definition    
        let struct_generics = base.generics.clone();
        let (_, ty_generics, _) = struct_generics.split_for_impl();
        
        // No special handling for specific types/methods - use trait methods as-is
        let trait_name = syn::Ident::new("Context", proc_macro2::Span::call_site());

        // Create the trait definition with the same generic parameters as the Context struct
        let struct_generics_params = generics.get_params();
        let struct_where_clause = generics.get_where_clause();
        
        // Use the same generic parameters for both the trait and struct
        let trait_def = quote::quote! {
            pub trait #trait_name<#(#struct_generics_params),*> #struct_where_clause {
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
                compiled_role.impl_block.self_ty = syn::parse_quote!(#type_name #ty_generics);
                compiled_role.impl_block.generics = generics.clone();
                compiled_role
            })
            .collect();

        // Create impl blocks that implement Context<T, ...>
        let context_methods = 
            blocks
                .iter()
                .map(|b| {
                    let mut impl_block = self.compile_context_methods(&roles_map, b.clone());
                    impl_block.generics = generics.clone();
                    
                    // Implement Context<T, ...> with the same generic parameters
                    let generic_args: Vec<syn::GenericArgument> = generics.get_params().iter().map(|param| {
                        match param {
                            syn::GenericParam::Type(tp) => syn::GenericArgument::Type(
                                syn::Type::Path(syn::TypePath {
                                    qself: None,
                                    path: syn::Path::from(tp.ident.clone()),
                                }),
                            ),
                            syn::GenericParam::Lifetime(l) => syn::GenericArgument::Lifetime(l.lifetime.clone()),
                            syn::GenericParam::Const(c) => syn::GenericArgument::Const(
                                syn::Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path::from(c.ident.clone()),
                                }),
                            ),
                        }
                    }).collect();
                    
                    let trait_with_generics = {
                        let mut segments = syn::punctuated::Punctuated::new();
                        let mut args = syn::punctuated::Punctuated::new();
                        
                        for arg in &generic_args {
                            args.push(arg.clone());
                        }
                        
                        let trait_segment = syn::PathSegment {
                            ident: trait_name.clone(),
                            arguments: if args.is_empty() {
                                syn::PathArguments::None
                            } else {
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: syn::Token![<](proc_macro2::Span::call_site()),
                                        args,
                                        gt_token: syn::Token![>](proc_macro2::Span::call_site()),
                                    }
                                )
                            }
                        };
                        
                        segments.push(trait_segment);
                        
                        syn::Path {
                            leading_colon: None,
                            segments,
                        }
                    };
                    
                    impl_block.implemented_traits = vec![trait_with_generics];
                    
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
                        
                        // Just preserve all existing generics as is - we'll handle 
                        // the correct generics at the impl block level
                        FunctionDescription::new_implementation(
                            name.clone(),
                            params.clone(),
                            generics.clone(),
                            output.clone(),
                            body,
                            asyncness.clone(),
                            attrs.clone()
                        )
                    }
                    f => f.clone(),
                }
            })
            .collect();

        // Compile the impl block
        let mut compiled = ImplBlockInfo {
            functions,
            generics: self.generics.clone(), // Use the struct generics directly
            ..impl_block
        }
        .compile();
        
        // Convert the struct generics to a format usable in the self_ty
        let generics = self.generics.to_syn_generics();
        let (_, type_generics, _) = generics.split_for_impl();
        
        // Create the self type with proper generics
        compiled.self_ty = {
            let mut tokens = proc_macro2::TokenStream::new();
            tokens.extend(quote::quote!(Context));
            tokens.extend(type_generics.to_token_stream());
            syn::parse2(tokens).unwrap()
        };
        
        compiled
    }

    fn compile_struct(&self) -> ItemStruct {
        // Start with the user-defined generics from the original Context struct
        let mut generics_params = Vec::new();
        
        // First add all of the user's original generic params - preserving their original bounds
        for param in self.generics.get_params() {
            generics_params.push(param.clone());
        }

        // Track which property types we need to generate generic parameters for
        let mut field_generics = Vec::new();

        // Map properties to their corresponding generic parameters or original types
        let property_generics: Vec<_> = self
            .properties
            .iter()
            .map(|prop| {
                if Self::is_primitive_type(&prop.get_ty()) {
                    // Primitive type: use original type
                    (prop.get_name().clone(), prop.get_ty().clone(), None)
                } else {
                    // Check if this type matches a trait role with a suffix like "Role"
                    let is_role = match &prop.get_ty() {
                        syn::Type::Path(type_path) if type_path.path.segments.len() == 1 => {
                            let segment = &type_path.path.segments[0];
                            segment.ident.to_string().ends_with("Role")
                        },
                        _ => false,
                    };

                    if is_role {
                        // Get the role type name
                        let role_type = match &prop.get_ty() {
                            syn::Type::Path(type_path) => type_path.path.segments[0].ident.clone(),
                            _ => panic!("Expected a Path type"),
                        };
                        
                        // Try to find the role in our roles list
                        let contract_ident = self.roles.iter()
                            .find(|r| r.name == role_type)
                            .map(|role| role.contract.name.clone())
                            .unwrap_or_else(|| {
                                // Fallback to naming convention if role or contract not found
                                let base_name = role_type.to_string();
                                let base_name = base_name.trim_end_matches("Role");
                                syn::Ident::new(&format!("{}Contract", base_name), proc_macro2::Span::call_site())
                            });
                            
                        let generic_name = syn::Ident::new(
                            &format!("T{}", to_upper_camel_case(&prop.get_name().to_string())),
                            proc_macro2::Span::call_site(),
                        );

                        // Add the field generic parameter to our tracking list
                        field_generics.push(generic_name.clone());

                        // Return the generic type instead of the original type
                        (
                            prop.get_name().clone(),
                            syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: syn::Path::from(generic_name),
                            }),
                            Some(contract_ident),
                        )
                    } else {
                        // Non-role type: keep as is
                        (prop.get_name().clone(), prop.get_ty().clone(), None)
                    }
                }
            })
            .collect();

        // Now add the role trait generic params to our list,
        // making sure we're not duplicating any that already exist
        for (idx, (_, _, contract_ident)) in property_generics.iter().enumerate() {
            if let Some(contract_ident) = contract_ident {
                let generic_name = &field_generics[idx];
                
                // Check if the parameter already exists
                let param_exists = generics_params.iter().any(|param| {
                    if let syn::GenericParam::Type(type_param) = param {
                        type_param.ident == *generic_name
                    } else {
                        false
                    }
                });

                if !param_exists {
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
                }
            }
        }

        // Generate fields for the struct
        let mut fields: Vec<syn::Field> = property_generics
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
            
        // Add PhantomData for all generic type parameters that aren't used in fields
        // This prevents "unused type parameter" warnings
        for param in &generics_params {
            if let syn::GenericParam::Type(tp) = param {
                let param_name = &tp.ident;
                
                // Check if this parameter is used in any field
                let used_in_field = fields.iter().any(|field| {
                    if let Some(_) = &field.ident {
                        if let syn::Type::Path(type_path) = &field.ty {
                            if let Some(segment) = type_path.path.segments.last() {
                                return segment.ident == *param_name;
                            }
                        }
                    }
                    false
                });
                
                if !used_in_field {
                    // Add a PhantomData field for this type parameter
                    let phantom_field = syn::Field {
                        attrs: vec![],
                        mutability: syn::FieldMutability::None, 
                        vis: syn::Visibility::Inherited,
                        ident: Some(syn::Ident::new(
                            &format!("_phantom_{}", param_name),
                            proc_macro2::Span::call_site(),
                        )),
                        colon_token: Some(Default::default()),
                        ty: syn::parse_quote!(::std::marker::PhantomData<#param_name>),
                    };
                    
                    fields.push(phantom_field);
                }
            }
        }

        // Preserve the original where clause, which is important for bounds like for<'de>
        let where_clause = self.generics.get_where_clause().clone();
        
        // Finalize generics with all parameters
        let mut generics = syn::Generics {
            lt_token: if generics_params.is_empty() { None } else { Some(Default::default()) },
            params: syn::punctuated::Punctuated::new(),
            gt_token: if generics_params.is_empty() { None } else { Some(Default::default()) },
            where_clause,
        };
        
        // Add all generics params
        for param in generics_params {
            generics.params.push(param);
        }

        // Construct the struct
        syn::ItemStruct {
            attrs: vec![],
            vis: syn::parse_quote!(pub),
            struct_token: syn::token::Struct {
                span: proc_macro2::Span::call_site(),
            },
            ident: self.name.clone(),
            generics,
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
