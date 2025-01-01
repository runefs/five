use syn::{Ident,Type};

#[derive(Clone)]
pub enum ParameterKind {
    ImmutableReference(ParameterInfo), // &T or &self
    MutableReference(ParameterInfo),   // &mut T or &mut self
    ByValue(ParameterInfo),            // T or self
}

#[derive(Clone)]
pub enum ParameterInfo {
    SelfRef, // Represents `self`, `&self`, or `&mut self`
    Typed {
        name: Ident, // Parameter name
        ty: Type,    // Parameter type
    },
}

pub fn analyze_parameters(sig: &syn::Signature) -> Vec<ParameterKind> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            // Handle `self`, `&self`, and `&mut self`
            syn::FnArg::Receiver(receiver) => {
                let kind = if receiver.reference.is_some() {
                    if receiver.mutability.is_some() {
                        ParameterKind::MutableReference(ParameterInfo::SelfRef)
                    } else {
                        ParameterKind::ImmutableReference(ParameterInfo::SelfRef)
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
                            ParameterKind::MutableReference(param_info)
                        } else {
                            ParameterKind::ImmutableReference(param_info)
                        }
                    } else {
                        ParameterKind::ByValue(param_info)
                    };
                    Some(kind)
                } else {
                    None
                }
            }
        })
        .collect()
}