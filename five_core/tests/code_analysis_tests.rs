mod data; // Relative path to `tests/mod/data.rs`

#[cfg(test)]
mod tests {

    use five_core::analysis::{analyze_impl_block, analyze_module, analyze_trait_methods, FunctionDescription, ParameterInfo, TypeDescription};
    use quote::ToTokens;
    use crate::data::*;

    #[test]
    fn test_analyze_trait_methods() {
        let item_trait: syn::ItemTrait = single_contract();

        let methods = analyze_trait_methods(&item_trait);

        assert_eq!(methods.len(), 2);

        // Validate `get_balance`
        let get_balance = &methods[0];
        match get_balance {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "get_balance");
                assert_eq!(params.len(), 1);

                // Validate `&self`
                match &params[0] {
                    ParameterInfo::ImmutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected ImmutableReference(Self) for `&self`"),
                }
            }
            _ => panic!("Expected Declaration for `get_balance`"),
        }

        // Validate `set_balance`
        let set_balance = &methods[1];
        match set_balance {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "set_balance");
                assert_eq!(params.len(), 2);

                // Validate `&mut self`
                match &params[0] {
                    ParameterInfo::MutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected MutableReference(Self) for `&mut self`"),
                }

                // Validate `value: i32`
                match &params[1] {
                    ParameterInfo::Typed { name, ty } => {
                        assert_eq!(name.to_string(), "value");
                        assert_eq!(ty.to_token_stream().to_string(), "i32");
                    }
                    _ => panic!("Expected ByValue(Typed) for `value`"),
                }
            }
            _ => panic!("Expected Declaration for `set_balance`"),
        }
    }
    #[test]
    fn test_role_with_contract() {
        let module: syn::ItemMod = simple_context();

        let analysis = analyze_module(&module);

        assert_eq!(analysis.context.roles.len(), 1);
        let role = &analysis.context.roles[0];
        assert_eq!(role.name.to_string(), "SourceRole");
        assert_eq!(role.contract.name.to_string(), "SourceContract");
    }
    #[test]
    fn test_analyze_module() {
        // Example `transfer` module as input
        let transfer_module: syn::ItemMod = transfer_context();

        // Perform the analysis
        let analysis = analyze_module(&transfer_module);
        // Validate roles
        assert_eq!(analysis.context.roles.len(), 2); // No roles ending in `Role` in the example
        
        // Validate role contracts
        assert!(analysis.context.roles.iter().any(|role| role.contract.name.to_string() == "SourceContract"));
        assert!(analysis.context.roles.iter().any(|role| role.contract.name.to_string() == "SinkContract"));
        // Validate `SourceContract`
        let source_role = analysis.context.roles.iter().find(|role| role.contract.name == "SourceContract").unwrap();
        assert_eq!(source_role.contract.functions.len(), 2);

        // Validate `get_balance` in `SourceContract`
        let get_balance = &source_role.contract.functions[0];
        match get_balance {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "get_balance");
                assert_eq!(params.len(), 1);

                // Validate `&self`
                match &params[0] {
                    ParameterInfo::ImmutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                }
            }
            _ => panic!("Expected Declaration for `get_balance`"),
        }

        // Validate `set_balance` in `SourceContract`
        let set_balance = &source_role.contract.functions[1];
        match set_balance {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "set_balance");
                assert_eq!(params.len(), 2);

                // Validate `&mut self`
                match &params[0] {
                    ParameterInfo::MutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected MutableReference(SelfRef) for `&mut self`"),
                }

                // Validate `value: i32`
                match &params[1] {
                    ParameterInfo::Typed { name, ty } => {
                        assert_eq!(name.to_string(), "value");
                        assert_eq!(ty.to_token_stream().to_string(), "i32");
                    }
                    _ => panic!("Expected ByValue(Typed) for `value`"),
                }
            }
            _ => panic!("Expected Declaration for `set_balance`"),
        }

        // Validate `SinkContract`
        let sink_role = analysis.context.roles.iter().find(|role| role.contract.name == "SinkContract").unwrap();
        assert_eq!(sink_role.contract.functions.len(), 1);

        // Validate `deposit` in `SinkContract`
        let deposit = &sink_role.contract.functions[0];
        match deposit {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "deposit");
                assert_eq!(params.len(), 2);

                // Validate `&mut self`
                match &params[0] {
                    ParameterInfo::MutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected MutableReference(SelfRef) for `&mut self`"),
                }

                // Validate `amount: i32`
                match &params[1] {
                    ParameterInfo::Typed { name, ty } => {
                        assert_eq!(name.to_string(), "amount");
                        assert_eq!(ty.to_token_stream().to_string(), "i32");
                    }
                    _ => panic!("Expected ByValue(Typed) for `amount`"),
                }
            }
            _ => panic!("Expected Declaration for `deposit`"),
        }

        // Validate context
        let context = analysis.context;
        assert_eq!(context.name.to_string(), "Context");
        assert_eq!(context.properties.len(), 3);
        assert!(context.properties.iter().any(|prop| prop.get_name() == "source"));
        assert!(context.properties.iter().any(|prop| prop.get_name() == "sink"));
        assert!(context.properties.iter().any(|prop| prop.get_name() == "amount"));

        // Validate other items
        assert_eq!(analysis.others.len(), 1); // The `impl Context` block
        let other = match &analysis.others[0] {
            TypeDescription::Other(syn::Item::Impl(impl_block)) => impl_block,
            _ => panic!("Expected Other with ItemImpl"),
        };
        assert_eq!(other.self_ty.to_token_stream().to_string(), "Context");
    }
    #[test]
    fn test_analyze_impl_block_with_generics_and_lifetime() {
        let item_impl: syn::ItemImpl = impl_with_generics();

        // Perform the analysis
        let impl_info = analyze_impl_block(&item_impl);

        // Validate the type being implemented
        assert_eq!(
            impl_info.self_ty.to_token_stream().to_string(),
            "Context < 'a , T >"
        );

        // Validate generics
        assert_eq!(impl_info.generics.get_params().len(), 2); // 'a and T
        assert!(matches!(
            impl_info.generics.get_params()[0],
            syn::GenericParam::Lifetime(_)
        ));
        assert!(matches!(
            impl_info.generics.get_params()[1],
            syn::GenericParam::Type(_)
        ));

        // Validate where clause
        let binding = impl_info.generics.get_where_clause();
        let where_clause = binding.as_ref().unwrap();
        assert_eq!(
            where_clause.predicates.to_token_stream().to_string(),
            "T : SomeTrait + AnotherTrait"
        );

        // Validate methods
        assert_eq!(impl_info.functions.len(), 2);

        // Validate `new`
        let new_method = &impl_info.functions[0];
        match new_method {
            FunctionDescription::Implementation { name, params, generics, .. } => {
                assert_eq!(name.to_string(), "new");
                assert_eq!(params.len(), 2); // value: T, data: &'a str

                // Validate `value: T`
                match &params[0] {
                    ParameterInfo::Typed { name, ty } => {
                        assert_eq!(name.to_string(), "value");
                        assert_eq!(ty.to_token_stream().to_string(), "T");
                    }
                    _ => panic!("Expected ByValue for `value`"),
                }

                // Validate `data: &'a str`
                match &params[1] {
                    ParameterInfo::Typed { name, ty } => {
                        assert_eq!(name.to_string(), "data");
                        assert_eq!(ty.to_token_stream().to_string(), "& 'a str");
                    }
                    _ => panic!("Expected ByValue for `data`"),
                }

                // Validate generics for the method
                assert_eq!(generics.get_params().len(), 0); // `new` has no extra generics
            }
            _ => panic!("Expected Implementation for `new`"),
        }

        // Validate `get_data`
        let get_data_method = &impl_info.functions[1];
        match get_data_method {
            FunctionDescription::Implementation { name, params, generics, .. } => {
                assert_eq!(name.to_string(), "get_data");
                assert_eq!(params.len(), 1); // Only &self

                // Validate `&self`
                match &params[0] {
                    ParameterInfo::ImmutableReference(inner) => match **inner {
                        ParameterInfo::SelfRef => {}
                        _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                    },
                    _ => panic!("Expected ImmutableReference(SelfRef) for `&self`"),
                }

                // Validate generics for the method
                assert_eq!(generics.get_params().len(), 0); // `get_data` has no extra generics
            }
            _ => panic!("Expected Implementation for `get_data`"),
        }
    }
}