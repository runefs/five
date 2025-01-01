use super::*;

#[derive(Clone)]
pub struct ImplBlockInfo {
    pub generics: GenericsInfo,
    pub for_lifetimes: Option<syn::Lifetime>,
    pub implemented_traits: Vec<syn::Path>,
    pub functions: Vec<FunctionDescription>,
}
impl ImplBlockInfo {
    pub fn new(
        generics: GenericsInfo,
        for_lifetimes: Option<syn::Lifetime>,
        implemented_traits: Vec<syn::Path>,
        functions: Vec<FunctionDescription>,
    ) -> Self {
        ImplBlockInfo {
            generics,
            for_lifetimes,
            implemented_traits,
            functions,
        }
    }
}