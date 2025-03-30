use super::*;
#[derive(Clone)]
pub enum TypeDescription {
    Role(TraitInfo),
    RoleContract(TraitInfo),
    Context(ContextInfo),
    Other(syn::Item),
}
