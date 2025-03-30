use std::collections::HashMap;

use super::{Compiled, CompiledImplBlock, CompiledTraitInfo, Compiler};
use crate::analysis::{FunctionDescription, ImplBlockInfo, Role, TraitInfo};
use syn::{visit_mut::VisitMut, Expr};

#[derive(Clone)]
pub struct CompiledRole {
    pub impl_block: CompiledImplBlock,
    pub contract: CompiledTraitInfo,
}

impl Compiled<Role> for CompiledRole {
    fn emit(&self) -> proc_macro2::TokenStream {
        use quote::quote;

        let impl_block = self.impl_block.emit();
        let contract = &self.contract;

        quote! {
            pub #contract

            #impl_block
        }
    }
}

pub fn to_snake_case(pascal_case: &str) -> String {
    let mut result = String::new();
    let mut chars = pascal_case.chars().peekable();

    while let Some(current) = chars.next() {
        if current.is_uppercase() {
            if !result.is_empty() &&
               // Check if previous char wasn't an underscore
               !result.ends_with('_') &&
               // Check if next char isn't uppercase (handles acronyms like "HTTP")
               !(chars.peek().map_or(false, |next| next.is_uppercase()))
            {
                result.push('_');
            }
            result.push(current.to_lowercase().next().unwrap());
        } else {
            result.push(current);
        }
    }

    result
}

pub(crate) fn to_role_name(trait_name: &str) -> String {
    let s = to_snake_case(trait_name);
    if s.ends_with("_role") {
        s[..s.len() - 5].to_owned()
    } else {
        s
    }
}

impl Role {
    pub fn compile(&self, roles: &HashMap<String, TraitInfo>) -> CompiledRole {
        let context_ty = syn::parse_str::<syn::Type>("Context_").unwrap();

        // Create a visitor to rewrite self to self.{role_name}
        struct SelfRewriter<'a> {
            role_name: syn::Ident,
            roles: &'a HashMap<String, TraitInfo>,
        }

        impl VisitMut for SelfRewriter<'_> {
            fn visit_expr_mut(&mut self, expr: &mut Expr) {
                if let Expr::Path(expr_path) = expr {
                    if expr_path.path.is_ident("self") {
                        let role_name = to_role_name(&self.role_name.to_string());
                        let role_ident = syn::Ident::new(&role_name, self.role_name.span());
                        *expr = syn::parse_quote!(self.#role_ident);
                        return;
                    }
                }
                if let Expr::MethodCall(method_call) = expr {
                    if let Expr::Path(base_path) = &*method_call.receiver {
                        if let Some(ident) = base_path.path.get_ident() {
                            if ident == "self" {
                                let role_trait =
                                    &self.roles[&to_role_name(&self.role_name.to_string())];
                                if !role_trait.functions.iter().any(|m| match m {
                                    FunctionDescription::Implementation { name, .. } => {
                                        name == &method_call.method
                                    }
                                    FunctionDescription::Declaration { name, .. } => {
                                        name == &method_call.method
                                    }
                                }) {
                                    // Create the new method name: role_method
                                    let new_method_name = syn::Ident::new(
                                        &format!(
                                            "{}_{}",
                                            &to_role_name(&self.role_name.to_string()),
                                            method_call.method
                                        ),
                                        method_call.method.span(),
                                    );

                                    // Create new method call with the same arguments
                                    let new_expr = Expr::MethodCall(syn::ExprMethodCall {
                                        attrs: method_call.attrs.clone(),
                                        receiver: Box::new(Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::parse_quote!(self),
                                        })),
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
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }

        // Compile the contract trait
        let contract = self.contract.compile();

        // Rewrite methods to access self.{role_name}
        let functions = self
            .methods
            .iter()
            .map(|func| match func {
                FunctionDescription::Implementation {
                    name,
                    params,
                    generics,
                    output,
                    body,
                } => {
                    let mut new_body = body.clone();
                    let mut rewriter = SelfRewriter {
                        role_name: self.name.clone(),
                        roles,
                    };
                    rewriter.visit_block_mut(&mut new_body);

                    let role_name = to_role_name(&self.name.to_string());
                    let new_name = syn::Ident::new(&format!("{}_{}", role_name, name), name.span());

                    FunctionDescription::new_implementation(
                        new_name,
                        params.clone(),
                        generics.clone(),
                        output.clone(),
                        new_body,
                    )
                }
                decl => decl.clone(),
            })
            .collect();

        let impl_block = ImplBlockInfo {
            self_ty: context_ty,
            generics: self.generics.clone(),
            for_lifetimes: None,
            implemented_traits: vec![],
            functions,
        };

        CompiledRole {
            impl_block: impl_block.compile(),
            contract,
        }
    }
}
