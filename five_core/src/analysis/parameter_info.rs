use quote::ToTokens;
use syn::{Ident, Lifetime, Type};

#[derive(Clone)]
pub enum ParameterInfo {
    SelfRef,                                // Represents `self`, `&self`, or `&mut self`
    ImmutableReference(Box<ParameterInfo>), // &T or &self
    MutableReference(Box<ParameterInfo>),   // &mut T or &mut self
    LifeTime(Lifetime, Box<ParameterInfo>), // Lifetime annotations
    Typed {
        name: Ident, // Parameter name
        ty: Type,    // Parameter type
    }, // T or self
}
impl ParameterInfo {
    fn inner(&self) -> &ParameterInfo {
        match self {
            ParameterInfo::ImmutableReference(pi) => pi.inner(),
            ParameterInfo::MutableReference(pi) => pi.inner(),
            ParameterInfo::SelfRef => self,
            ParameterInfo::Typed { name: _, ty: _ } => self,
            ParameterInfo::LifeTime(..) => panic!("Recursive Lifetime should not happen"),
        }
    }
    pub fn name(&self) -> String {
        match self.inner() {
            ParameterInfo::SelfRef => "self".to_string(),
            ParameterInfo::Typed { name, ty: _ } => name.to_string(),
            _ => panic!("Should have been removed in inner()"),
        }
    }

    pub fn is_self(&self) -> bool {
        matches!(self.inner(), ParameterInfo::SelfRef)
    }

    pub fn get_self_type(&self) -> SelfType {
        match self {
            ParameterInfo::SelfRef => SelfType::Value,
            ParameterInfo::ImmutableReference(inner)
                if matches!(**inner, ParameterInfo::SelfRef) =>
            {
                SelfType::Reference
            }
            ParameterInfo::MutableReference(inner) if matches!(**inner, ParameterInfo::SelfRef) => {
                SelfType::MutableReference
            }
            _ => panic!("Called get_self_type on non-self parameter"),
        }
    }

    pub fn new_owned(name: &str, ty: syn::Type) -> Self {
        ParameterInfo::Typed {
            name: syn::Ident::new(name, proc_macro2::Span::call_site()),
            ty,
        }
    }

    pub fn new_ref(name: &str, ty: syn::Type) -> Self {
        ParameterInfo::ImmutableReference(Box::new(ParameterInfo::Typed {
            name: syn::Ident::new(name, proc_macro2::Span::call_site()),
            ty,
        }))
    }

    pub fn new_mut_ref(name: &str, ty: syn::Type) -> Self {
        ParameterInfo::MutableReference(Box::new(ParameterInfo::Typed {
            name: syn::Ident::new(name, proc_macro2::Span::call_site()),
            ty,
        }))
    }
}

pub fn analyze_parameters(sig: &syn::Signature) -> Vec<ParameterInfo> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            // Handle `self`, `&self`, and `&mut self`
            syn::FnArg::Receiver(receiver) => {
                let kind = if receiver.reference.is_some() {
                    if receiver.mutability.is_some() {
                        ParameterInfo::MutableReference(Box::new(ParameterInfo::SelfRef))
                    } else {
                        ParameterInfo::ImmutableReference(Box::new(ParameterInfo::SelfRef))
                    }
                } else {
                    panic!("Self should be passed by reference or not be present")
                };
                Some(kind)
            }

            // Handle typed parameters
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    let param_info = ParameterInfo::Typed {
                        name: pat_ident.ident.clone(),
                        ty: *pat_type.ty.clone(),
                    };

                    let kind = if pat_ident.by_ref.is_some() {
                        if pat_ident.mutability.is_some() {
                            ParameterInfo::MutableReference(Box::new(param_info))
                        } else {
                            ParameterInfo::ImmutableReference(Box::new(param_info))
                        }
                    } else {
                        param_info
                    };
                    Some(kind)
                } else {
                    None
                }
            }
        })
        .collect()
}

#[derive(Debug)]
pub enum SelfType {
    Value,
    Reference,
    MutableReference,
}

impl ToTokens for ParameterInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ParameterInfo::SelfRef => tokens.extend(quote::quote!(self)),
            ParameterInfo::ImmutableReference(inner) => {
                tokens.extend(quote::quote!(&));
                inner.to_tokens(tokens);
            }
            ParameterInfo::MutableReference(inner) => {
                tokens.extend(quote::quote!(&mut));
                inner.to_tokens(tokens);
            }
            ParameterInfo::LifeTime(lifetime, inner) => {
                lifetime.to_tokens(tokens);
                tokens.extend(quote::quote!(:));
                inner.to_tokens(tokens);
            }
            ParameterInfo::Typed { name, ty } => {
                name.to_tokens(tokens);
                tokens.extend(quote::quote!(:));
                ty.to_tokens(tokens);
            }
        }
    }
}
