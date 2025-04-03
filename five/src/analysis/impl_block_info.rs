use super::*;
use syn::{Item, Type};

#[derive(Clone)]
pub struct ImplBlockInfo {
    pub generics: GenericsInfo,
    pub for_lifetimes: Option<syn::Lifetime>,
    pub implemented_traits: Vec<syn::Path>,
    pub functions: Vec<FunctionDescription>,
    pub self_ty: Type,
    pub attrs: Vec<syn::Attribute>,
}
impl ImplBlockInfo {
    pub fn new(
        generics: GenericsInfo,
        for_lifetimes: Option<syn::Lifetime>,
        implemented_traits: Vec<syn::Path>,
        functions: Vec<FunctionDescription>,
        self_ty: Type,
        attrs: Vec<syn::Attribute>,
    ) -> Self {
        ImplBlockInfo {
            generics,
            for_lifetimes,
            implemented_traits,
            functions,
            self_ty,
            attrs,
        }
    }
}
pub fn analyze_impl_block(item_impl: &syn::ItemImpl) -> ImplBlockInfo {
    // Analyze generics
    let generics = analyze_generics(&Item::Impl(item_impl.clone()));
    let self_ty = (*item_impl.self_ty).clone();
    // Analyze for<> lifetimes
    let for_lifetimes = item_impl.generics.params.iter().find_map(|param| {
        if let syn::GenericParam::Lifetime(lifetime_def) = param {
            Some(lifetime_def.lifetime.clone())
        } else {
            None
        }
    });

    // Analyze implemented traits
    let implemented_traits = item_impl
        .trait_
        .as_ref()
        .map_or(Vec::new(), |(_, path, _)| vec![path.clone()]);

    // Analyze methods in the impl block
    let functions = item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                let params = analyze_parameters(&method.sig);
                let generics = analyze_generics_from_impl_method(method);
                let body = method.block.clone();

                Some(FunctionDescription::Implementation {
                    name: method.sig.ident.clone(),
                    params,
                    generics,
                    output: method.sig.output.clone(),
                    body,
                    asyncness: method.sig.asyncness.clone(),
                    attrs: method.attrs.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    // Create and return the ImplBlockInfo
    ImplBlockInfo::new(
        generics,
        for_lifetimes,
        implemented_traits,
        functions,
        self_ty,
        item_impl.attrs.clone(),
    )
}
