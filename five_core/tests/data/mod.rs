use syn::{ItemImpl, ItemMod, ItemTrait};

pub(crate) fn single_contract() -> ItemTrait { 
    syn::parse_quote! {
        pub trait SourceContract {
            fn get_balance(&self) -> i32;
            fn set_balance(&mut self, value: i32);
        }
    }
}



pub(crate) fn simple_context() -> ItemMod {
    syn::parse_quote! {
        mod test_module {
            pub trait SourceContract {
                fn get_balance(&self) -> i32;
            }

            pub trait SourceRole: SourceContract {}

            struct Context{

            } impl Context {
                pub fn name() -> &str{ "hello" }
            }
        }
    }
}

pub(crate) fn transfer_context() -> ItemMod {
    syn::parse_quote! {
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
    }
}

pub(crate) fn impl_with_generics() -> ItemImpl {
    syn::parse_quote! {
        impl<'a, T: SomeTrait + AnotherTrait> Context<'a, T> {
            pub fn new(value: T, data: &'a str) -> Self {
                Context { value, data }
            }

            pub fn get_data(&self) -> &'a str {
                self.data
            }
        }
    }
}