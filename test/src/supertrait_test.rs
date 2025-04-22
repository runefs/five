#[five::context]
pub mod supertrait_test {
    // Define a contract trait
    pub trait DataContract {
        fn process(&self, data: &str) -> String;
    }
    
    // Define a role that directly extends the contract
    // Notice we're not using the naming convention here
    trait ProcessorRole: DataContract {
        fn transform(&self, input: &str) -> String {
            let processed = self.process(input);
            format!("Transformed: {}", processed)
        }
    }
    
    // Context struct with the role
    struct Context {
        processor: ProcessorRole
    }
    
    // Implementation with methods that call the role methods
    impl Context {
        pub fn process_data(&self, input: &str) -> String {
            self.processor.transform(input)
        }
    } 
} 