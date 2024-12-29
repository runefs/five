use five::context;

#[context]
pub mod transfer { 
    pub trait SourceContract<'a> {
        fn get_balance(&self) -> i32;
        fn get_account_no(&self) -> i64;
        fn add_withdrawl(&mut self, message: &'a str, value: i32);
    }

    pub trait SinkContract<'a> {
        fn get_balance(&self) -> i32;
        fn get_account_no(&self) -> i64;
        fn add_deposit(&mut self, message: &'a str, value: i32);
    }

    trait SourceRole<'a> : SourceContract<'a> {
        fn withdraw(&mut self, message: &'a str, value: i32) {
            let balance = self.get_balance();
            if value > balance {
                panic!("Insufficient funds")
            }
            self.add_withdrawl(message,value);
            println!("Withdrew: {}. New balance is {}", value, self.get_balance());
        }
    }

    trait SinkRole<'a> : SinkContract<'a> {
        fn deposit(&mut self, value: i32) {
            let new_balance = self.get_balance() + value;
            self.set_balance(new_balance);
            println!("Deposited: {}. New balance is {}", value, new_balance);
        }
    }

    struct Context<'a> {
        source: SourceRole<'a>,
        sink: SinkRole<'a>,
        amount: i32
    }
    impl for<'a> Context<'a>  {
        fn transfer(&mut self, from_message: &'a str, to_message: &'a str){
            self.source.withdraw(from_message, self.amount);
            self.sink.deposit(to_message, self.amount);
        }
    }
}