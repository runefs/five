use syn::Ident;

use super::*;

#[derive(Clone)]
pub struct ModuleInfo {
    pub module_name: Ident,
    pub context: ContextInfo,
    pub others: Vec<TypeDescription>,
}

pub fn analyze_module(module: &syn::ItemMod) -> ModuleInfo {
    let mut roles = Vec::new();
    let mut contexts = Vec::new();
    let mut others = Vec::new();
    let module_name = module.ident.clone();
    if let Some((_, items)) = &module.content {
        let mut contracts = Vec::new();
        let mut impl_blocks_by_type: std::collections::HashMap<String, Vec<syn::ItemImpl>> = std::collections::HashMap::new();

        // Separate contracts and roles for association
        for item in items {
            match item {
                syn::Item::Trait(item_trait) => {
                    if item_trait.ident.to_string().ends_with("Contract") {
                        contracts.push(analyze_trait(item_trait));
                    }
                }
                syn::Item::Impl(item_impl) => {
                    // Get the self type of the impl block
                    if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            let type_name = segment.ident.to_string();
                            impl_blocks_by_type
                                .entry(type_name)
                                .or_default()
                                .push(item_impl.clone());
                        }
                    }
                }
                _ => (),
            }
        }

        for item in items {
            match item {
                syn::Item::Trait(item_trait) => {
                    if item_trait.ident.to_string().ends_with("Role") {
                        let role_name = item_trait.ident.to_string();
                        let contract_name = role_name.strip_suffix("Role").unwrap_or("").to_string()
                            + "Contract";

                        // Find the matching contract
                        let contract = contracts.iter().find(|contract| {
                            contract.name == syn::Ident::new(&contract_name, contract.name.span())
                        });

                        if let Some(contract) = contract {  
                            let role = Role {
                                name: item_trait.ident.clone(),
                                contract: contract.clone(),
                                generics: analyze_generics(item),
                                methods: analyze_trait_methods(item_trait),
                            };
                            roles.push(role);
                        } else {
                            panic!("No matching contract found for role: {}", role_name);
                        }
                    }
                }
                syn::Item::Struct(item_struct)=> {
                    if item_struct.ident.to_string() == "Context" {
                        let impl_blocks = impl_blocks_by_type
                            .get(&item_struct.ident.to_string())
                            .cloned()
                            .unwrap_or_else(Vec::new);
                        contexts.push(analyze_context(item_struct, &impl_blocks));
                        if contexts.len() != 1 {
                            panic!("There should be exactly one Context struct");
                        }
                    }
                }
                syn::Item::Impl(item_impl) => {
                    // Check if the `impl` is for the `Context` type
                    if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            if segment.ident == "Context" {
                                // Skip adding to `others`
                                continue;
                            }
                        }
                    }
                    others.push(TypeDescription::Other(item.clone()));
                }
                _ => {
                    others.push(TypeDescription::Other(item.clone()));
                }
            }
        }
    }
    if contexts.len() != 1 {
        panic! ("There should be exactly one Context struct. Found {}", contexts.len());
    }
    let mut context = contexts[0].clone();
    context.roles = roles;

    ModuleInfo {
        module_name,
        context,
        others
    }
}