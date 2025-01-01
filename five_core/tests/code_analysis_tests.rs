#[cfg(test)]
mod tests {
    use five_core::analysis::*;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_analyze_trait_methods() {
        let item_trait: syn::ItemTrait = syn::parse_quote! {
            pub trait SourceContract {
                fn get_balance(&self) -> i32;
                fn set_balance(&mut self, value: i32);
            }
        };

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
                    ParameterKind::ImmutableReference(ParameterInfo::SelfRef) => {}
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
                    ParameterKind::MutableReference(ParameterInfo::SelfRef) => {}
                    _ => panic!("Expected MutableReference(Self) for `&mut self`"),
                }

                // Validate `value: i32`
                match &params[1] {
                    ParameterKind::ByValue(ParameterInfo::Typed { name, ty }) => {
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
        let module: syn::ItemMod = syn::parse_quote! {
            mod test_module {
                pub trait SourceContract {
                    fn get_balance(&self) -> i32;
                }

                pub trait SourceRole: SourceContract {}

                struct Context{}
            }
        };

        let analysis = analyze_module(&module);

        assert_eq!(analysis.roles.len(), 1);
        let role = &analysis.roles[0];
        assert_eq!(role.name.to_string(), "SourceRole");
        assert_eq!(role.contract.name.to_string(), "SourceContract");
    }
    #[test]
    fn test_analyze_module() {
        // Example `transfer` module as input
        let transfer_module: syn::ItemMod = parse_quote! {
            mod transfer {
                pub trait SourceContract {
                    fn get_balance(&self) -> i32;
                    fn set_balance(&mut self, value: i32);
                }

                pub trait SinkContract {
                    fn deposit(&mut self, amount: i32);
                }

                trait SourceRole : SourceContract {
                    fn withdraw(&mut self, value: i32) {
                        let balance = self.get_balance();
                        if value > balance {
                            panic!("Insufficient funds")
                        }
                        println!("Withdrew: {}. New balance is {}", value, self.get_balance());
                    }
                }

                trait SinkRole : SinkContract {
                    fn deposit(&mut self, value: i32) {
                        let new_balance = self.get_balance() + value;
                        self.set_balance(new_balance);
                        println!("Deposited: {}. New balance is {}", value, new_balance);
                    }
                }

                pub struct Context {
                    source: Box<dyn SourceContract>,
                    sink: Box<dyn SinkContract>,
                    amount: i32,
                }

                impl Context {
                    pub fn transfer(&self) {
                        self.sink.deposit(self.amount);
                    }
                }
            }
        };

        // Perform the analysis
        let analysis = analyze_module(&transfer_module);

        // Validate roles
        assert_eq!(analysis.roles.len(), 2); // No roles ending in `Role` in the example
        
        // Validate role contracts
        assert!(analysis.roles.iter().any(|role| role.contract.name.to_string() == "SourceContract"));
        assert!(analysis.roles.iter().any(|role| role.contract.name.to_string() == "SinkContract"));

        // Validate `SourceContract`
        let source_role = analysis.roles.iter().find(|role| role.contract.name == "SourceContract").unwrap();
        assert_eq!(source_role.contract.functions.len(), 2);

        // Validate `get_balance` in `SourceContract`
        let get_balance = &source_role.contract.functions[0];
        match get_balance {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "get_balance");
                assert_eq!(params.len(), 1);

                // Validate `&self`
                match &params[0] {
                    ParameterKind::ImmutableReference(ParameterInfo::SelfRef) => {}
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
                    ParameterKind::MutableReference(ParameterInfo::SelfRef) => {}
                    _ => panic!("Expected MutableReference(SelfRef) for `&mut self`"),
                }

                // Validate `value: i32`
                match &params[1] {
                    ParameterKind::ByValue(ParameterInfo::Typed { name, ty }) => {
                        assert_eq!(name.to_string(), "value");
                        assert_eq!(ty.to_token_stream().to_string(), "i32");
                    }
                    _ => panic!("Expected ByValue(Typed) for `value`"),
                }
            }
            _ => panic!("Expected Declaration for `set_balance`"),
        }

        // Validate `SinkContract`
        let sink_role = analysis.roles.iter().find(|role| role.contract.name == "SinkContract").unwrap();
        assert_eq!(sink_role.contract.functions.len(), 1);

        // Validate `deposit` in `SinkContract`
        let deposit = &sink_role.contract.functions[0];
        match deposit {
            FunctionDescription::Declaration { name, params, .. } => {
                assert_eq!(name.to_string(), "deposit");
                assert_eq!(params.len(), 2);

                // Validate `&mut self`
                match &params[0] {
                    ParameterKind::MutableReference(ParameterInfo::SelfRef) => {}
                    _ => panic!("Expected MutableReference(SelfRef) for `&mut self`"),
                }

                // Validate `amount: i32`
                match &params[1] {
                    ParameterKind::ByValue(ParameterInfo::Typed { name, ty }) => {
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
}