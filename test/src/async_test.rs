use async_trait::async_trait;

#[five::context]
pub mod async_test {
    #[async_trait]
    pub trait TestContract {
        async fn test(&self) -> Result<String, String>;
    }

    struct Context {
        test: TestContract,
    }

    impl Context {
        async fn test(&self) -> Result<String, String> {
            self.test.test().await
        }
    }
} 