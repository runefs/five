use five::context;
pub enum LedgerEntry<'a> {
    Deposit(&'a str, i32),
    Withdrawal(&'a str, i32)
}

#[context]
pub mod account { 
     
    use super::LedgerEntry;

    trait LedgerContract : {
        fn push(&mut self, entry: LedgerEntry);
        fn as_vec(&self) -> Vec<LedgerEntry>;
    }

    trait LedgerRole : LedgerContract{
        fn add(&mut self, entry: LedgerEntry){
            self.push(entry);
        }
    }

    struct Context {
        ledger : LedgerRole,
        accout_no: i64
    }
    impl Context {
        fn deposit(&self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Deposit(message,amount))
        }
        fn withdraw(&self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Withdrawal(message,amount))
        }
    }
}