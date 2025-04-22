use crate::data::data::UserProfile;

#[five::context]
pub mod user_profile_context {
    
    #[async_trait::async_trait]
    pub trait UserStorageContract {
        async fn store(&self, key: String, data: UserProfile) -> Result<String, String>;
        async fn retrieve(&self, key: String) -> Result<UserProfile, String>;
    }
    
    pub trait UserStorageRole: UserStorageContract {
    }
    
    pub struct Context {
        user_storage: UserStorageRole
    }
    
    impl Context {
        pub async fn store_profile(&self, profile: &UserProfile) -> Result<String, String> {
            self.user_storage.store(profile.sub().to_string(), profile.clone()).await
        }
        
        pub async fn retrieve_profile(&self, sub: &str) -> Result<UserProfile, String> {
            self.user_storage.retrieve(sub.to_string()).await
        }
    }
}