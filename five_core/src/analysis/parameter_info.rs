use syn::{Ident, Lifetime, Type};
use quote::ToTokens;

#[derive(Clone)]
pub enum ParameterInfo {
    SelfRef,                                // Represents `self`, `&self`, or `&mut self`
    ImmutableReference(Box<ParameterInfo>), // &T or &self
    MutableReference(Box<ParameterInfo>),   // &mut T or &mut self
    LifeTime(Lifetime, Box<ParameterInfo>), // Lifetime annotations
    Typed {
        name: Ident, // Parameter name
        ty: Type,    // Parameter type
    },               // T or self
}
impl<'a> ParameterInfo {
    fn inner(&self) -> &ParameterInfo {
        match self {
            ParameterInfo::ImmutableReference(pi) => {
                pi.inner()
            },
            ParameterInfo::MutableReference(pi) => {
                pi.inner()
            },
            ParameterInfo::SelfRef => {
                self
            },
            ParameterInfo::Typed { name: _, ty: _ } => {
                self
            },
            ParameterInfo::LifeTime(..) => panic!("Recursive Lifetime should not happen")
        }
    }
    pub fn name(&self) -> String {
        match self.inner() {
            ParameterInfo::SelfRef => {
                "self".to_string()
            },
            ParameterInfo::Typed { name, ty: _ } => {
                name.to_string()
            },
            _ => panic!("Should have been removed in inner()")
        }
    }

    pub fn is_self(&self) -> bool {
        matches!(self.inner(), ParameterInfo::SelfRef)
    }

    pub fn get_self_type(&self) -> SelfType {
        match self {
            ParameterInfo::SelfRef => SelfType::Value,
            ParameterInfo::ImmutableReference(inner) if matches!(**inner, ParameterInfo::SelfRef) => SelfType::Reference,
            ParameterInfo::MutableReference(inner) if matches!(**inner, ParameterInfo::SelfRef) => SelfType::MutableReference,
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

    pub fn to_syn_param(&self) -> syn::FnArg {
        match self {
            ParameterInfo::SelfRef => syn::FnArg::Receiver(syn::Receiver {
                attrs: vec![],
                reference: None,
                mutability: None,
                self_token: syn::Token![self](proc_macro2::Span::call_site()),
                colon_token: None,
                ty: Box::new(syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path::from(syn::Ident::new("Self", proc_macro2::Span::call_site())),
                })),
            }),
            ParameterInfo::ImmutableReference(inner) if matches!(&**inner, ParameterInfo::SelfRef) => {
                syn::FnArg::Receiver(syn::Receiver {
                    attrs: vec![],
                    reference: Some((syn::Token![&](proc_macro2::Span::call_site()), None)),
                    mutability: None,
                    self_token: syn::Token![self](proc_macro2::Span::call_site()),
                    colon_token: None,
                    ty: Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path::from(syn::Ident::new("Self", proc_macro2::Span::call_site())),
                    })),
                })
            },
            ParameterInfo::MutableReference(inner) if matches!(&**inner, ParameterInfo::SelfRef) => {
                syn::FnArg::Receiver(syn::Receiver {
                    attrs: vec![],
                    reference: Some((syn::Token![&](proc_macro2::Span::call_site()), None)),
                    mutability: Some(syn::Token![mut](proc_macro2::Span::call_site())),
                    self_token: syn::Token![self](proc_macro2::Span::call_site()),
                    colon_token: None,
                    ty: Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path::from(syn::Ident::new("Self", proc_macro2::Span::call_site())),
                    })),
                })
            },
            ParameterInfo::Typed { name, ty } => {
                syn::FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: name.clone(),
                        subpat: None,
                    })),
                    colon_token: syn::Token![:](proc_macro2::Span::call_site()),
                    ty: Box::new(ty.clone()),
                })
            },
            ParameterInfo::ImmutableReference(inner) => {
                if let syn::FnArg::Typed(mut pat_type) = inner.to_syn_param() {
                    pat_type.pat = Box::new(syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: Some(syn::Token![ref](proc_macro2::Span::call_site())),
                        mutability: None,
                        ident: match &*pat_type.pat {
                            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                            _ => panic!("Expected ident pattern"),
                        },
                        subpat: None,
                    }));
                    syn::FnArg::Typed(pat_type)
                } else {
                    panic!("Expected typed parameter");
                }
            },
            ParameterInfo::MutableReference(inner) => {
                if let syn::FnArg::Typed(mut pat_type) = inner.to_syn_param() {
                    pat_type.pat = Box::new(syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: Some(syn::Token![ref](proc_macro2::Span::call_site())),
                        mutability: Some(syn::Token![mut](proc_macro2::Span::call_site())),
                        ident: match &*pat_type.pat {
                            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                            _ => panic!("Expected ident pattern"),
                        },
                        subpat: None,
                    }));
                    syn::FnArg::Typed(pat_type)
                } else {
                    panic!("Expected typed parameter");
                }
            },
            ParameterInfo::LifeTime(lifetime, inner) => {
                // For lifetime parameters, we'll need to modify the type to include the lifetime
                if let syn::FnArg::Typed(mut pat_type) = inner.to_syn_param() {
                    // Add lifetime to the type
                    // Note: This is a simplified approach and might need to be enhanced
                    // depending on how you want to handle lifetimes
                    pat_type.ty = Box::new(Type::Reference(syn::TypeReference {
                        and_token: syn::Token![&](proc_macro2::Span::call_site()),
                        lifetime: Some(lifetime.clone()),
                        mutability: None,
                        elem: pat_type.ty,
                    }));
                    syn::FnArg::Typed(pat_type)
                } else {
                    panic!("Expected typed parameter");
                }
            },
        }
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

                    let kind = if let Some(_) = pat_ident.by_ref {
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
            ParameterInfo::SelfRef => {
                tokens.extend(quote::quote!(self))
            }
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