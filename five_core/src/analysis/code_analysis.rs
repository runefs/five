use quote::ToTokens;

use super::*;

pub struct CodeAnalysis {
    pub roles: Vec<Role>,
    pub context: ContextInfo,
    pub others: Vec<TypeDescription>,
}

pub fn analyze_module(module: &syn::ItemMod) -> CodeAnalysis {
    let mut roles = Vec::new();
    let mut contexts = Vec::new();
    let mut others = Vec::new();

    if let Some((_, items)) = &module.content {
        let mut contracts = Vec::new();

        // Separate contracts and roles for association
        for item in items {
            match item {
                syn::Item::Trait(item_trait) => {
                    if item_trait.ident.to_string().ends_with("Contract") {
                        contracts.push(analyze_trait(item_trait));
                    }
                }
                _ => (),
            }
        }

        for item in items {
            print!("Item: {:#?}",item.to_token_stream());
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
                                generics: analyze_generics(&item_trait.generics),
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
                        contexts.push(analyze_context(item_struct));
                        if contexts.len() != 1 {
                            panic!("There should be exactly one Context struct");
                        }
                    }
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

    CodeAnalysis {
        roles, // No independent contracts anymore
        context: contexts[0].clone(),
        others,
    }
}