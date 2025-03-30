use syn::Item;

use super::*;

#[derive(Clone)]
pub struct TraitInfo {
    pub name: syn::Ident,
    pub generics: GenericsInfo,
    pub functions: Vec<FunctionDescription>,
}
#[allow(dead_code)]
impl TraitInfo {
    pub fn new(
        name: syn::Ident,
        generics: GenericsInfo,
        functions: Vec<FunctionDescription>,
    ) -> Self {
        TraitInfo {
            name,
            generics,
            functions,
        }
    }
}

pub fn analyze_trait(item_trait: &syn::ItemTrait) -> TraitInfo {
    let generics = analyze_generics(&Item::Trait(item_trait.clone()));
    let functions = item_trait
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                Some(FunctionDescription::Declaration {
                    name: method.sig.ident.clone(),
                    params: analyze_parameters(&method.sig),
                    generics: analyze_generics_from_method(method),
                    output: method.sig.output.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    TraitInfo {
        name: item_trait.ident.clone(),
        generics,
        functions,
    }
}
