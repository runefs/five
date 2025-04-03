mod account;
mod storage;
use account::LedgerContract;

fn main() {
    test_account();
    test_storage();
}

fn test_storage() {
    struct InMemoryStore {
        data: HashMap<String, Vec<u8>>,
    }
    impl storage::StoreContract for InMemoryStore {
        fn store(&self, key: String, data: Vec<u8>) -> Result<String, String> {
            self.data.insert(key, data);
            Ok(key)
        }
        fn retrieve(&self, key: String) -> Result<Vec<u8>, String> {
            self.data.get(&key).ok_or("Key not found".to_string())
        }
    }
}

fn test_account() {
    struct Aa {
        vec: Vec<account::LedgerEntry>,
    }

    impl Aa {
        fn new() -> Self {
            Self { vec: Vec::new() }
        }
    }
    impl LedgerContract for Aa {
        fn push(&mut self, entry: account::LedgerEntry) {
            self.vec.push(entry);
        }
        fn as_vec(&self) -> Vec<account::LedgerEntry> {
            self.vec.clone()
        }
    }
    let ledger = Aa::new();

    use account::Account;
    let mut account = account::bind(ledger, 67676555);

    account.deposit(String::from("Deposit 1"), 100);
    account.withdraw(String::from("Withdrawal 1"), 50);
    account.deposit(String::from("Deposit 2"), 200);
    account.withdraw(String::from("Withdrawal 2"), 100);
    println!("Balance: {}", account.balance()); //access to role contract methods incorrectly rewritten as if it was a role method access
    assert_eq!(account.balance(), 150);
    //println!("Balance: {}", account.balance());
}
