#[five::context]
pub mod storage { 
    
    pub struct NoEncrypter {
        
    }
    impl EncrypterContract for Encrypter {
        fn encrypt(&self, data: Vec<u8>) -> Vec<u8> {
            data
        }
        fn decrypt(&self, data: Vec<u8>) -> Vec<u8> {
            data
        }
    }
    pub trait SerializerContract : {
        fn serialize(&self, data: String) -> Vec<u8>;
        fn deserialize(&self, data: Vec<u8>) -> String;
    }

    pub trait EncrypterContract : {
        fn encrypt(&self, data: Vec<u8>) -> Vec<u8>;
        fn decrypt(&self, data: Vec<u8>) -> Vec<u8>;
    }

    pub trait StorageContract : {
        fn store(&self,key: String, data: Vec<u8>) -> String;
        fn retrieve(&self, key: String) -> Vec<u8>;
    }

    trait  SerializerRole :  SerializerContract{ }
    trait  EncrypterRole :  EncrypterContract{ }
    trait  StorageRole :  StorageContract{ }
    struct Context {
        serialiser : SerialiserRole,
        encrypter: EncrypterRole,
        storage: StorageRole,
    }
    impl Context {
        fn store(&mut self, key: String, data: String)-> String {
            let serialised = self.serialiser.serialize(data);
            let encrypted = self.encrypter.encrypt(serialised);
            self.storage.store(key,encrypted)
        } 
        fn retrieve(&mut self, key: String){
            let encrypted = self.storage.retrieve(data);
            let decrypted = self.encrypter.decrypt(encrypted);
            let deserialised = self.serialiser.deserialize(decrypted);
            deserialised
        }
    }
}