use super::*;

#[derive(Clone)]
pub struct Role {
    pub name: syn::Ident,                  // The role's name (e.g., `SourceRole`)
    pub contract: TraitInfo,               // Associated contract (e.g., `SourceContract`)
    pub generics: GenericsInfo,            // Generics for the role
    pub methods: Vec<FunctionDescription>, // Methods specific to the role
}
