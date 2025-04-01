use quote::ToTokens;
use syn::{Block, Ident, ReturnType};

use crate::analysis::{FunctionDescription, GenericsInfo, ParameterInfo};

use super::{Compiled, Compiler};

#[allow(dead_code)]
#[derive(Clone)]
pub enum CompiledFunctionDescription {
    Declaration {
        name: Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        output: ReturnType,
        asyncness: Option<syn::token::Async>,
        attrs: Vec<syn::Attribute>,
    },
    Implementation {
        name: Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        output: ReturnType,
        body: Option<Block>,
        asyncness: Option<syn::token::Async>,
        attrs: Vec<syn::Attribute>,
    },
}

impl Compiled<CompiledFunctionDescription> for CompiledFunctionDescription {
    fn emit(&self) -> proc_macro2::TokenStream {
        use quote::quote;

        match self {
            CompiledFunctionDescription::Implementation {
                name,
                params,
                generics,
                output,
                body,
                asyncness,
                attrs,
            } => {
                // Convert parameters to TokenStream
                let mut inputs = syn::punctuated::Punctuated::new();

                // Split params into self and non-self parameters
                let (self_param, other_params): (Vec<_>, Vec<_>) =
                    params.iter().partition(|p| p.is_self());

                // Add self parameter first if it exists
                if let Some(self_param) = self_param.first() {
                    let tokens = self_param.to_token_stream();
                    let arg: syn::FnArg = syn::parse2(tokens).unwrap();
                    inputs.push_value(arg);
                    inputs.push_punct(syn::Token![,](proc_macro2::Span::call_site()));
                }

                // Add remaining parameters
                for param in other_params {
                    let tokens = param.to_token_stream();
                    let arg: syn::FnArg = syn::parse2(tokens).unwrap();
                    inputs.push_value(arg);
                    inputs.push_punct(syn::Token![,](proc_macro2::Span::call_site()));
                }

                // Create the function signature
                let sig = syn::Signature {
                    constness: None,
                    asyncness: asyncness.clone(),
                    unsafety: None,
                    abi: None,
                    fn_token: syn::token::Fn::default(),
                    ident: name.clone(),
                    generics: generics.to_syn_generics(),
                    paren_token: syn::token::Paren::default(),
                    inputs,
                    variadic: None,
                    output: output.clone(),
                };

                // Create the function item
                let item_fn = syn::ItemFn {
                    attrs: attrs.clone(),
                    vis: syn::Visibility::Public(syn::token::Pub::default()),
                    sig,
                    block: Box::new(body.clone().unwrap_or_else(|| syn::Block {
                        brace_token: syn::token::Brace::default(),
                        stmts: vec![],
                    })),
                };

                quote!(#item_fn)
            }
            CompiledFunctionDescription::Declaration { .. } => proc_macro2::TokenStream::new(),
        }
    }
}

impl Compiler<CompiledFunctionDescription> for FunctionDescription {
    fn compile(&self) -> CompiledFunctionDescription {
        match self {
            FunctionDescription::Declaration {
                name,
                params,
                generics,
                output,
                asyncness,
                attrs,
            } => CompiledFunctionDescription::Declaration {
                name: name.clone(),
                params: params.clone(),
                generics: generics.clone(),
                output: output.clone(),
                asyncness: asyncness.clone(),
                attrs: attrs.clone(),
            },
            FunctionDescription::Implementation {
                name,
                params,
                generics,
                output,
                body,
                asyncness,
                attrs,
            } => CompiledFunctionDescription::Implementation {
                name: name.clone(),
                params: params.clone(),
                generics: generics.clone(),
                output: output.clone(),
                body: Some(body.clone()),
                asyncness: asyncness.clone(),
                attrs: attrs.clone(),
            },
        }
    }

    type Output = CompiledFunctionDescription;
}
