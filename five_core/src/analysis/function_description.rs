use super::*;
use syn::{Ident, Block};
#[derive(Clone)]
pub enum FunctionDescription {
    Declaration {
        name: Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
    },
    Implementation {
        name: Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        body: Block,
    },
}
impl FunctionDescription {
    pub fn new_declaration(
        name: syn::Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
    ) -> Self {
        FunctionDescription::Declaration {
            name,
            params,
            generics,
        }
    }

    pub fn new_implementation(
        name: syn::Ident,
        params: Vec<ParameterInfo>,
        generics: GenericsInfo,
        body: syn::Block,
    ) -> Self {
        FunctionDescription::Implementation {
            name,
            params,
            generics,
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

                // Determine if the method has a default implementation
                if let Some(body) = &method.default {
                    Some(FunctionDescription::Implementation {
                        name: method.sig.ident.clone(),
                        params,
                        generics,
                        body: body.clone(),
                    })
                } else {
                    Some(FunctionDescription::Declaration {
                        name: method.sig.ident.clone(),
                        params,
                        generics,
                    })
                }
            } else {
                None
            }
        })
        .collect()
}