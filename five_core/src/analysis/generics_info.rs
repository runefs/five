use syn::{GenericParam, WhereClause};

#[derive(Clone)]
pub struct GenericsInfo {
    params: Vec<GenericParam>,
    where_clause: Option<WhereClause>,
}
impl GenericsInfo {
    pub fn new(
        params: Vec<syn::GenericParam>,
        where_clause: Option<syn::WhereClause>,
    ) -> Self {
        GenericsInfo { params, where_clause }
    }
    pub fn get_params(&self) -> Vec<GenericParam> {
        self.params.clone()
    }
    pub fn get_where_clause(&self) -> Option<WhereClause> {
        self.where_clause.clone()
    }

}


pub fn analyze_generics(generics: &syn::Generics) -> GenericsInfo {
    GenericsInfo {
        params: generics.params.clone().into_iter().collect(),
        where_clause: generics.where_clause.clone(),
    }
}