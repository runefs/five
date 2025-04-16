mod account;
mod storage;
use std::collections::HashMap;
use std::sync::Mutex;
use account::LedgerContract;
use lazy_static::lazy_static;
use storage::{SerialiserType, StoreContract, EncrypterContract,SerialiserContract};
use crate::storage::Storage;
use serde::{Serialize,Deserialize};
lazy_static! {
    static ref GLOBAL_STORAGE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() {
    test_account();
    test_storage().await;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub value: String
}

async fn test_storage() {
    struct Serialiser;
    impl SerialiserContract for Serialiser {
        fn get_type(&self) -> SerialiserType {
            SerialiserType::Cbor
        }
    }

    struct Encrypter;
    impl EncrypterContract for Encrypter {
        fn get_key(&self) -> &[u8] {
            b"01234567890123456789012345678901"
        }
    }
    
    struct InMemoryStore;
    
    #[async_trait::async_trait]
    impl StoreContract for InMemoryStore {
        async fn store(&self, key: String, data: Vec<u8>) -> Result<String, String> {
            println!("Storing data for key: {}", key);
            let mut storage = GLOBAL_STORAGE.lock().map_err(|e| e.to_string())?;
            storage.insert(key.clone(), data);
            Ok(key)
        }
        async fn retrieve(&self, key: String) -> Result<Vec<u8>, String> {
            println!("Retrieving data for key: {}", key);
            let storage = GLOBAL_STORAGE.lock().map_err(|e| e.to_string())?;
            let v = storage.get(&key).ok_or("Key not found".to_string())?;
            Ok(v.clone())
        }
    }


    let store = storage::bind::<Data, Serialiser, Encrypter, InMemoryStore>(Serialiser, Encrypter,InMemoryStore);
    let key = "FirstKey";
    let data = Data { value: "a lot of very important data".to_string() };
    store.store(key.to_string(), &data).await.unwrap();
    let data = store.retrieve(key.to_string()).await.unwrap();
    println!("Data: {:?}", data);
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
