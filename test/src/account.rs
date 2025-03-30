#[five::context]
pub mod account {
    #[derive(Debug, Clone)]
    pub enum LedgerEntry {
        Deposit(String, i32),
        Withdrawal(String, i32),
    }

    impl LedgerEntry {
        fn message(&self) -> String {
            match self {
                LedgerEntry::Deposit(msg, _) => msg.to_string(),
                LedgerEntry::Withdrawal(msg, _) => msg.to_string(),
            }
        }
    }

    pub trait LedgerContract {
        fn push(&mut self, entry: LedgerEntry);
        fn as_vec(&self) -> Vec<LedgerEntry>;
    }

    trait LedgerRole: LedgerContract {
        fn add(&mut self, entry: LedgerEntry) {
            self.push(entry.clone());
            self.log(entry.message());
        }
        fn log(&self, msg: String) {
            println!("{}", msg);
        }
    }

    struct Context {
        ledger: LedgerRole,
        account_no: i64,
    }
    impl Context {
        fn deposit(&mut self, message: String, amount: i32) {
            self.ledger.add(LedgerEntry::Deposit(message, amount))
        }
        fn withdraw(&mut self, message: String, amount: i32) {
            self.ledger.add(LedgerEntry::Withdrawal(message, amount))
        }

        fn balance(&self) -> i32 {
            self.ledger
                .as_vec()
                .iter()
                .map(|entry| match entry {
                    LedgerEntry::Deposit(_, amount) => *amount,
                    LedgerEntry::Withdrawal(_, amount) => -*amount,
                })
                .sum()
        }
    }
}
