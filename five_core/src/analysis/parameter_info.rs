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
    },               // T or self
}
impl<'a> ParameterInfo {
    pub fn name(&self) -> String {
        match self {
            ParameterInfo::ImmutableReference(pi) => {
                pi.name()
            }
            ParameterInfo::MutableReference(pi) => {
                pi.name()
            },
            ParameterInfo::SelfRef => {
                "self".to_string()
            },
            ParameterInfo::Typed { name, ty: _ } => {
                name.to_string()
            },
            ParameterInfo::LifeTime(..) => panic!("Recursive Lifetime should not happen")
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