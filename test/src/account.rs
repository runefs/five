use five::context;
pub enum LedgerEntry<'a> {
    Deposit(&'a str, i32),
    Withdrawal(&'a str, i32)
}

#[context]
pub mod account { 
    
    use super::LedgerEntry;

    trait LedgerContract<'a> : {
        fn push(&mut self, entry: LedgerEntry<'a>);
        fn as_vec(&self) -> Vec<LedgerEntry<'a>>;
    }

    trait LedgerRole<'a> : LedgerContract<'a>{
        fn add(&mut self, entry: LedgerEntry){
            self.push(entry);
        }
    }

    struct Context<'a> {
        ledger : LedgerRole<'a>,
        accout_no: i64
    }
    impl for<'a> Context<'a> {
        fn deposit(&self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Deposit(message,amount))
        }
        fn withdraw(&self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Withdrawal(message,amount))
        }
    }
}