use syn::Item;

use super::*;

#[derive(Clone)]
pub struct TraitInfo {
    pub name: syn::Ident,
    pub generics: GenericsInfo,
    pub functions: Vec<FunctionDescription>,
    pub attrs: Vec<syn::Attribute>,
}

pub fn analyze_trait(item_trait: &syn::ItemTrait) -> TraitInfo {
    let generics = analyze_generics(&Item::Trait(item_trait.clone()));
    let functions = item_trait
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                let params = analyze_parameters(&method.sig);
                let output = method.sig.output.clone();
                Some(FunctionDescription::Declaration {
                    name: method.sig.ident.clone(),
                    params,
                    generics: analyze_generics_from_method(method),
                    output,
                    asyncness: method.sig.asyncness.clone(),
                    attrs: method.attrs.clone(),
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
        attrs: item_trait.attrs.clone(),
    }
}
