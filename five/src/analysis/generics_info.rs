use quote::ToTokens;
use syn::{GenericParam, Generics, ImplItemFn, TraitItemFn, WhereClause};

#[derive(Clone)]
pub struct GenericsInfo {
    params: Vec<GenericParam>,
    where_clause: Option<WhereClause>,
}
impl GenericsInfo {
    pub fn new(params: Vec<syn::GenericParam>, where_clause: Option<syn::WhereClause>) -> Self {
        GenericsInfo {
            params,
            where_clause,
        }
    }
    pub fn get_params(&self) -> Vec<GenericParam> {
        self.params.clone()
    }
    pub fn get_where_clause(&self) -> Option<WhereClause> {
        self.where_clause.clone()
    }
}

impl ToTokens for GenericsInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Only add angle brackets if we have parameters
        if !self.params.is_empty() {
            tokens.extend(quote::quote!(<));

            // Add each parameter with commas between
            let mut first = true;
            for param in &self.params {
                if !first {
                    tokens.extend(quote::quote!(,));
                }
                param.to_tokens(tokens);
                first = false;
            }

            tokens.extend(quote::quote!(>));
        }

        // Add where clause if present
        if let Some(where_clause) = &self.where_clause {
            where_clause.to_tokens(tokens);
        }
    }
}

impl GenericsInfo {
    pub fn from_syn_generics(generics: &syn::Generics) -> Self {
        GenericsInfo {
            params: generics.params.clone().into_iter().collect(),
            where_clause: generics.where_clause.clone(),
        }
    }
}

pub fn analyze_generics(item: &syn::Item) -> GenericsInfo {
    if let (Some(generics), where_clause) = extract_inline_where_clauses(item) {
        analyze_generics_(generics, where_clause)
    } else {
        GenericsInfo {
            params: Vec::new(),
            where_clause: None,
        }
    }
}
pub fn analyze_generics_from_method(method: &TraitItemFn) -> GenericsInfo {
    analyze_generics_(
        &method.sig.generics,
        extract_inline_where_clauses_from_signature(&method.sig),
    )
}

pub fn analyze_generics_from_impl_method(method: &ImplItemFn) -> GenericsInfo {
    analyze_generics_(
        &method.sig.generics,
        extract_inline_where_clauses_from_signature(&method.sig),
    )
}

fn analyze_generics_(
    generics: &syn::Generics,
    additional_where_clauses: Vec<syn::WherePredicate>,
) -> GenericsInfo {
    // Start with the existing where clause, if any
    let mut where_clause = generics.where_clause.clone();

    // Merge additional where clauses into the existing where clause
    if !additional_where_clauses.is_empty() {
        where_clause = merge_where_clause(where_clause, additional_where_clauses);
    }

    GenericsInfo {
        params: generics.params.clone().into_iter().collect(),
        where_clause,
    }
}

fn merge_where_clause(
    existing_where_clause: Option<syn::WhereClause>,
    new_clauses: Vec<syn::WherePredicate>,
) -> Option<syn::WhereClause> {
    match existing_where_clause {
        Some(mut wc) => {
            wc.predicates.extend(new_clauses);
            Some(wc)
        }
        None => Some(syn::WhereClause {
            where_token: Default::default(),
            predicates: new_clauses.into_iter().collect(),
        }),
    }
}

fn extract_inline_where_clauses(item: &syn::Item) -> (Option<&Generics>, Vec<syn::WherePredicate>) {
    match item {
        syn::Item::Impl(item_impl) => (
            Some(&item_impl.generics),
            extract_inline_where_clauses_from_impl(item_impl),
        ),
        syn::Item::Trait(item_trait) => (
            Some(&item_trait.generics),
            extract_inline_where_clauses_from_trait(item_trait),
        ),
        _ => (None, Vec::new()),
    }
}

fn extract_inline_where_clauses_from_impl(item_impl: &syn::ItemImpl) -> Vec<syn::WherePredicate> {
    // Start with the generics from the impl block
    let mut where_clauses: Vec<syn::WherePredicate> = item_impl
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                Some(syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: type_param.ident.clone().into(),
                    }),
                    colon_token: Default::default(),
                    bounds: type_param.bounds.clone(),
                }))
            } else {
                None
            }
        })
        .collect();

    // Add any existing where clause predicates
    if let Some(where_clause) = &item_impl.generics.where_clause {
        where_clauses.extend(where_clause.predicates.clone());
    }

    where_clauses
}

fn extract_inline_where_clauses_from_trait(
    item_trait: &syn::ItemTrait,
) -> Vec<syn::WherePredicate> {
    item_trait
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                Some(syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: type_param.ident.clone().into(),
                    }),
                    colon_token: Default::default(),
                    bounds: type_param.bounds.clone(),
                }))
            } else {
                None
            }
        })
        .collect()
}

pub fn extract_inline_where_clauses_from_signature(
    sig: &syn::Signature,
) -> Vec<syn::WherePredicate> {
    sig.generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                Some(syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: type_param.ident.clone().into(),
                    }),
                    colon_token: Default::default(),
                    bounds: type_param.bounds.clone(),
                }))
            } else {
                None
            }
        })
        .collect()
}
