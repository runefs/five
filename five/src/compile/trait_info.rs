use quote::ToTokens;

use super::{Compiled, Compiler};
use crate::analysis::{FunctionDescription, TraitInfo};

#[derive(Clone)]
pub struct CompiledTraitInfo {
    pub trait_item: syn::ItemTrait,
}

impl ToTokens for CompiledTraitInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.trait_item.to_tokens(tokens);
    }
}

impl Compiled<TraitInfo> for CompiledTraitInfo {
    fn emit(&self) -> proc_macro2::TokenStream {
        self.to_token_stream()
    }
}

impl Compiler<TraitInfo> for TraitInfo {
    fn compile(&self) -> CompiledTraitInfo {
        let functions = self
            .functions
            .iter()
            .map(|f| {
                match f {
                    FunctionDescription::Declaration {
                        name,
                        params,
                        generics,
                        output,
                        asyncness,
                    } => {
                        let param_tokens = params.iter().map(|p| p.to_token_stream());
                        let generic_params = generics.get_params();
                        let where_clause = generics.get_where_clause();

                        // Only add angle brackets if we have generic parameters
                        let generic_tokens = if !generic_params.is_empty() {
                            quote::quote!(<#(#generic_params),*>)
                        } else {
                            quote::quote!()
                        };

                        // Add async keyword if needed
                        if asyncness.is_some() {
                            syn::parse_quote! {
                                async fn #name #generic_tokens (#(#param_tokens),*) #output #where_clause;
                            }
                        } else {
                            syn::parse_quote! {
                                fn #name #generic_tokens (#(#param_tokens),*) #output #where_clause;
                            }
                        }
                    }
                    FunctionDescription::Implementation { .. } => {
                        panic!("Trait should only contain declarations")
                    }
                }
            })
            .collect::<Vec<syn::TraitItem>>();

        let trait_item = syn::ItemTrait {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            unsafety: None,
            auto_token: None,
            trait_token: syn::token::Trait::default(),
            ident: self.name.clone(),
            generics: self.generics.to_syn_generics(),
            colon_token: None,
            supertraits: syn::punctuated::Punctuated::new(),
            brace_token: syn::token::Brace::default(),
            items: functions,
            restriction: None,
        };

        CompiledTraitInfo { trait_item }
    }

    type Output = CompiledTraitInfo;
}
