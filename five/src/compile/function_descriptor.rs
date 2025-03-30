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
    },
    Implementation {
        name: Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        output: ReturnType,
        body: Block,
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
                    asyncness: None,
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
                    attrs: vec![],
                    vis: syn::Visibility::Public(syn::token::Pub::default()),
                    sig,
                    block: Box::new(body.clone()),
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
            } => CompiledFunctionDescription::Declaration {
                name: name.clone(),
                params: params.clone(),
                generics: generics.clone(),
                output: output.clone(),
            },
            FunctionDescription::Implementation {
                name,
                params,
                generics,
                output,
                body,
            } => CompiledFunctionDescription::Implementation {
                name: name.clone(),
                params: params.clone(),
                generics: generics.clone(),
                output: output.clone(),
                body: body.clone(),
            },
        }
    }

    type Output = CompiledFunctionDescription;
}
