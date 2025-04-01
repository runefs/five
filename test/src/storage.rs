use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;


#[allow(dead_code)]
pub enum SerialiserType {
    Json,
    Cbor
}

#[five::context]
pub mod storage {
    pub trait SerialiserContract : {
        fn get_type(&self) -> SerialiserType;
    }


    pub trait StoreContract: Send + Sync {
        fn store(&self, key: String, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
        fn retrieve(&self, key: String) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + '_>>;
    }
    pub trait EncrypterContract {
        fn get_key(&self) -> &[u8];
    }

    trait StoreRole :  StoreContract{ 
       
    }

    trait SerialiserRole :  SerialiserContract{ 
        fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, String> {
            match self.get_type() {
                SerialiserType::Json => serde_json::to_vec(data).map_err(|e| format!("JSON serialization error: {}", e)),
                SerialiserType::Cbor => serde_cbor::to_vec(data).map_err(|e| format!("CBOR serialization error: {}", e))
            }
        }
        
        fn deserialize<T: for<'de> Deserialize<'de>>(&self,data: Vec<u8>) -> Result<T, String> {
            match self.get_type() {
                SerialiserType::Json => serde_json::from_slice(&data).map_err(|e| format!("JSON deserialization error: {}", e)),
                SerialiserType::Cbor => serde_cbor::from_slice(&data).map_err(|e| format!("CBOR deserialization error: {}", e))
            }
        }
    }
    trait EncrypterRole : EncrypterContract {
        fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            let cipher = Aes256Gcm::new_from_slice(self.get_key())
                .map_err(|_| "Invalid encryption key".to_string())?;
            
            let nonce_bytes: [u8; 12] = rand::random();
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let encrypted_data = cipher.encrypt(nonce, data)
                .map_err(|e| format!("Encryption error: {}", e))?;
            
            let mut result = nonce_bytes.to_vec();
            result.extend(encrypted_data);
            Ok(result)
        }
        
        fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            if data.len() < 12 {
                return Err("Invalid data: too short".to_string());
            }
            
            let cipher = Aes256Gcm::new_from_slice(self.get_key())
                .map_err(|_| "Invalid encryption key".to_string())?;
            
            let nonce = Nonce::from_slice(&data[..12]);
            let encrypted_content = &data[12..];
            
            cipher.decrypt(nonce, encrypted_content)
                .map_err(|_| "Decryption failed".to_string())
        }
    }
    struct Context {
        serialiser : SerialiserRole,
        encrypter: EncrypterRole,
        store: StoreRole
    }
    impl Context {
        #[inline]
        fn should_encrypt(&self) -> bool {
            #[cfg(release)]
            {
                true
            }
            #[cfg(debug_assertions)]
            {
                false
            }
        }
        async fn store<T: Serialize>(&self, key: String, data: &T)-> Result<String, String> {
            let serialised = self.serialiser.serialize(data)?;
            let encrypted = if self.should_encrypt() {
                self.encrypter.encrypt(serialised.as_slice())?
            } else {
                serialised
            };
            self.store.store(key, encrypted).await
        } 
        async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: String) -> Result<T, String>{
            let encrypted_data = self.store.retrieve(key).await?;
            let decrypted = if self.should_encrypt() {
                self.encrypter.decrypt(encrypted_data.as_slice())?
            } else {
                encrypted_data
            };
            self.serialiser.deserialize::<T>(decrypted)
        }
    }
}