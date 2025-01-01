use super::*;

#[derive(Clone)]
pub struct ContextInfo {
    pub name: syn::Ident,
    pub generics: GenericsInfo,
    pub properties: Vec<PropertyInfo>,
    pub impl_blocks: Vec<ImplBlockInfo>,
}
impl ContextInfo {
    pub fn new(
        name: syn::Ident,
        generics: GenericsInfo,
        properties: Vec<PropertyInfo>,
        impl_blocks: Vec<ImplBlockInfo>,
    ) -> Self {
        ContextInfo {
            name,
            generics,
            properties,
            impl_blocks,
        }
    }
}

pub fn analyze_context(item_struct: &syn::ItemStruct) -> ContextInfo {
    let generics = analyze_generics(&item_struct.generics);
    let properties = item_struct
        .fields
        .iter()
        .map(|field| PropertyInfo::new(
            field.ident.clone().unwrap(),
            field.ty.clone()
        ))
        .collect();

    ContextInfo {
        name: item_struct.ident.clone(),
        generics,
        properties,
        impl_blocks: Vec::new(),
    }
}