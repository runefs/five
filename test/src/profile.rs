use crate::data::data::UserProfile;

#[five::context]
mod user_profile_context {
    
    #[async_trait::async_trait]
    trait UserStorageContract {
        async fn store(&self, key: String, data: UserProfile) -> Result<String, String>;
        async fn retrieve(&self, key: String) -> Result<UserProfile, String>;
    }
    trait UserStorageRole: UserStorageContract {

    }
    
    struct Context {
        user_storage: UserStorageRole
    }
    
    impl Context {
        async fn store_profile(&self, profile: &UserProfile) -> Result<String, String> {
            // Store logic here
            self.storage.store(profile.sub().to_string(), serde_json::to_vec(profile).map_err(|e| e.to_string())?).await
        }
        
        async fn retrieve_profile(&self, sub: &str) -> Result<UserProfile, String> {
            let data = self.storage.retrieve(sub.to_string()).await?;
            serde_json::from_slice(&data).map_err(|e| e.to_string())
        }
    }
}