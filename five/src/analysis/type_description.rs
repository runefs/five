use super::*;
#[allow(dead_code)]
#[derive(Clone)]
pub enum TypeDescription {
    Role(TraitInfo),
    RoleContract(TraitInfo),
    Context(ContextInfo),
    Other(syn::Item),
}
