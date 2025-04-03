use super::*;
use quote::ToTokens;
#[allow(dead_code)]
#[derive(Clone)]
pub struct ContextInfo {
    pub name: syn::Ident,
    pub generics: GenericsInfo,
    pub properties: Vec<PropertyInfo>,
    pub impl_blocks: Vec<ImplBlockInfo>,
    pub roles: Vec<Role>,
    pub attrs: Vec<syn::Attribute>,
}
#[allow(dead_code)]
impl ContextInfo {
    pub fn new(
        name: syn::Ident,
        generics: GenericsInfo,
        properties: Vec<PropertyInfo>,
        impl_blocks: Vec<ImplBlockInfo>,
        roles: Vec<Role>,
        attrs: Vec<syn::Attribute>,
    ) -> Self {
        ContextInfo {
            name,
            generics,
            properties,
            impl_blocks,
            roles,
            attrs,
        }
    }
}

pub fn analyze_context(
    item_struct: &syn::ItemStruct,
    impl_blocks: &[syn::ItemImpl],
) -> ContextInfo {
    let generics = analyze_generics(&syn::Item::Struct(item_struct.clone()));
    let properties = item_struct
        .fields
        .iter()
        .map(|field| PropertyInfo::new(field.ident.clone().unwrap(), field.ty.clone()))
        .collect();

    // Analyze the provided impl blocks directly
    let analyzed_impl_blocks = impl_blocks.iter().map(analyze_impl_block).collect();
    
    ContextInfo {
        name: item_struct.ident.clone(),
        generics,
        properties,
        impl_blocks: analyzed_impl_blocks,
        roles: vec![],
        attrs: item_struct.attrs.clone(),
    }
}
