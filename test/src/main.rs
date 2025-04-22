mod account;
mod storage;
mod user_profile_context;
mod data;
use std::collections::HashMap;
use std::sync::Mutex;
use account::LedgerContract;
use data::data::UserProfile;
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
    test_user_profile_context().await;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub value: String
}
struct Serialiser;
struct Encrypter;
struct InMemoryStore;
impl SerialiserContract for Serialiser {
    fn get_type(&self) -> SerialiserType {
        SerialiserType::Cbor
    }
}


impl EncrypterContract for Encrypter {
    fn get_key(&self) -> &[u8] {
        b"01234567890123456789012345678901"
    }
}

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

fn create_storage_context<T : Serialize + for<'de> Deserialize<'de>>() -> impl crate::storage::Storage<T,Serialiser,Encrypter,InMemoryStore> {
    storage::bind::<T, Serialiser, Encrypter, InMemoryStore>(Serialiser, Encrypter,InMemoryStore)
}


async fn test_storage() {
    let store = create_storage_context::<Data>();
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

#[async_trait::async_trait]
impl crate::user_profile_context::UserStorageContract for crate::storage::Context<UserProfile,Serialiser,Encrypter,InMemoryStore> {
    async fn store(&self, key: String, data: UserProfile) -> Result<String, String> {
        <Self as Storage<UserProfile, _, _, _>>::store(self, key, &data).await
    }
    async fn retrieve(&self, key: String) -> Result<UserProfile, String> {
        <Self as Storage<UserProfile, _, _, _>>::retrieve(self, key).await
    }
}

async fn test_user_profile_context() {
    let serialiser = Serialiser;
    let encrypter = Encrypter;
    let store = InMemoryStore;
    
    // Create a concrete storage context
    let storage = storage::bind::<UserProfile, _, _, _>(serialiser, encrypter, store);
    
    // Cast to concrete type
    let concrete_storage: crate::storage::Context<UserProfile,Serialiser,Encrypter,InMemoryStore> = unsafe { 
        std::mem::transmute(storage) 
    };
    
    // Import the traits we need
    use crate::user_profile_context::UserProfileContext;
    
    let context = crate::user_profile_context::bind(concrete_storage);
    
    // Create a user profile with the required 'sub' field
    let user_profile = UserProfile::new("user123".to_string())
        .with_name("John Doe".to_string())
        .with_email("john.doe@example.com".to_string());
    
    // Use the public methods from UserProfileContext
    context.store_profile(&user_profile).await.unwrap();
    println!("Stored profile for: {}", user_profile.sub());
    
    let retrieved = context.retrieve_profile(user_profile.sub()).await.unwrap();
    println!("Retrieved profile: {:?}", retrieved);
}
