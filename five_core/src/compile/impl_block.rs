
use crate::analysis::{GenericsInfo, ImplBlockInfo};

use super::{function_descriptor::CompiledFunctionDescription, Compiled, Compiler};

#[derive(Clone)]
pub struct CompiledImplBlock {
    pub generics: GenericsInfo,
    pub for_lifetimes: Option<syn::Lifetime>,
    pub implemented_traits: Vec<syn::Path>,
    pub functions: Vec<CompiledFunctionDescription>,
    pub self_ty: syn::Type
}

impl Compiled<ImplBlockInfo> for CompiledImplBlock {
    fn emit(&self) -> proc_macro2::TokenStream {
        use quote::quote;
        
        let functions = self.functions.iter().map(|func| {
            // If we're implementing a trait, strip the pub modifier
            if !self.implemented_traits.is_empty() {
                // Convert to string, remove "pub ", and parse back
                let tokens = func.emit().to_string().replace("pub ", "");
                tokens.parse().unwrap()
            } else {
                func.emit()
            }
        });

        let impl_generics = &self.generics.get_params();
        let impl_where = &self.generics.get_where_clause();
        
        // Split the generic parameters into just the type parameters without bounds
        let type_params = impl_generics.iter().map(|param| {
            match param {
                syn::GenericParam::Type(t) => &t.ident,
                _ => panic!("Unexpected generic parameter type")
            }
        });
        
        // Only add angle brackets if we have generic parameters
        let impl_generic_tokens = if !impl_generics.is_empty(){
            quote!(<#(#impl_generics),*>)
        } else {
            quote!()
        };

        let type_generic_tokens = if !impl_generics.is_empty() {
            quote!(<#(#type_params),*>)
        } else {
            quote!()
        };

        // Handle implemented traits
        let impl_trait = if !self.implemented_traits.is_empty() {
            let traits = &self.implemented_traits;
            quote!(#(#traits)+* for)
        } else {
            quote!()
        };
        
        quote! {
            impl #impl_generic_tokens #impl_trait Context #type_generic_tokens #impl_where {
                #(#functions)*
            }
        }
    }
}

impl Compiler<ImplBlockInfo> for ImplBlockInfo {
    fn compile(&self) -> CompiledImplBlock {
        CompiledImplBlock {
            generics: self.generics.clone(),
            for_lifetimes: self.for_lifetimes.clone(),
            implemented_traits: self.implemented_traits.clone(),
            functions: self.functions.iter().map(|f| f.compile()).collect(),
            self_ty: self.self_ty.clone(),
        }
    }
    
    type Output = CompiledImplBlock;
}
