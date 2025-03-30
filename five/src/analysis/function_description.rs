use super::*;
use syn::{Block, Ident, ReturnType};
#[derive(Clone)]
pub enum FunctionDescription {
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
impl FunctionDescription {
    pub fn get_name(&self) -> &Ident {
        match self {
            Self::Declaration { name, .. } => name,
            Self::Implementation { name, .. } => name,
        }
    }
    pub fn get_params(&self) -> &Vec<ParameterInfo> {
        match self {
            Self::Declaration {
                name: _, params, ..
            } => params,
            Self::Implementation {
                name: _, params, ..
            } => params,
        }
    }
    pub fn get_generics(&self) -> &GenericsInfo {
        match self {
            Self::Declaration {
                name: _,
                params: _,
                generics,
                ..
            } => generics,
            Self::Implementation {
                name: _,
                params: _,
                generics,
                ..
            } => generics,
        }
    }
    pub fn get_output(&self) -> &ReturnType {
        match self {
            Self::Declaration {
                name: _,
                params: _,
                generics: _,
                output,
            } => output,
            Self::Implementation {
                name: _,
                params: _,
                generics: _,
                output,
                ..
            } => output,
        }
    }

    pub fn new_declaration(
        name: syn::Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        output: ReturnType,
    ) -> Self {
        FunctionDescription::Declaration {
            name,
            params,
            generics,
            output,
        }
    }

    pub fn new_implementation(
        name: syn::Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        output: ReturnType,
        body: syn::Block,
    ) -> Self {
        FunctionDescription::Implementation {
            name,
            params,
            generics,
            output,
            body,
        }
    }
}
pub fn analyze_trait_methods(item_trait: &syn::ItemTrait) -> Vec<FunctionDescription> {
    item_trait
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                // Analyze method parameters
                let params = analyze_parameters(&method.sig);

                // Analyze method generics
                let generics = analyze_generics_from_method(method);
                let output = method.sig.output.clone();
                // Determine if the method has a default implementation
                if let Some(body) = &method.default {
                    Some(FunctionDescription::Implementation {
                        name: method.sig.ident.clone(),
                        params,
                        generics,
                        output,
                        body: body.clone(),
                    })
                } else {
                    Some(FunctionDescription::Declaration {
                        name: method.sig.ident.clone(),
                        params,
                        generics,
                        output,
                    })
                }
            } else {
                None
            }
        })
        .collect()
}
