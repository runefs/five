#[five::context]
pub mod account { 
    pub enum LedgerEntry<'a> {
        Deposit(&'a str, i32),
        Withdrawal(&'a str, i32)
    } 
    impl<'a> Clone for LedgerEntry<'a> {
        fn clone(&self) -> Self {
            match self {
                LedgerEntry::Deposit(msg, amount) => LedgerEntry::Deposit(msg, *amount),
                LedgerEntry::Withdrawal(msg, amount) => LedgerEntry::Withdrawal(msg, *amount),
            }
        }
    }

    impl<'a> LedgerEntry<'a> {
        fn message(&self) -> &str {
            match self
            {
                LedgerEntry::Deposit(msg, _) => msg,
                LedgerEntry::Withdrawal(msg, _) => msg,
            }
        }
    }

    trait LedgerContract : {
        fn push(&mut self, entry: LedgerEntry);
        fn as_vec(&self) -> Vec<LedgerEntry>;
    }

    trait LedgerRole : LedgerContract{
        fn add(&mut self, entry: LedgerEntry){
            self.push(entry.clone());
            self.log(entry.message());
        }
        fn log(&self, msg: &str) {
            println!("{}",msg);
        }
    }

    struct Context {
        ledger : LedgerRole,
        account_no: i64
    }
    impl Context {
        fn deposit(&mut self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Deposit(message,amount))
        }
        fn withdraw(&mut self, message : &str, amount: i32){
            self.ledger.add(LedgerEntry::Withdrawal(message,amount))
        }
    }
}