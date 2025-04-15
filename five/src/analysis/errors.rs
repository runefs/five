use proc_macro2::Span;
use std::fmt;
use syn::Error as SynError;

/// Result type specialized for proc macro errors
pub type Result<T> = std::result::Result<T, MacroError>;

/// Custom error type that stores the span information for better error reporting
#[derive(Debug)]

pub struct MacroError {
    /// The error message
    message: String,
    /// The span where the error occurred
    span: Option<Span>,
    /// Optional source error
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}
#[allow(dead_code)]
impl MacroError {
    /// Create a new error with a message and span
    pub fn new<T: fmt::Display>(message: T, span: Span) -> Self {
        MacroError {
            message: message.to_string(),
            span: Some(span),
            source: None,
        }
    }

    /// Create a new error with only a message
    pub fn without_span<T: fmt::Display>(message: T) -> Self {
        MacroError {
            message: message.to_string(),
            span: None,
            source: None,
        }
    }

    /// Set the span of the error
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Get the span of the error
    pub fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    /// Add a source error
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Convert this error to a syn::Error, which can be used to emit compiler diagnostics
    pub fn to_syn_error(&self) -> SynError {
        match &self.span {
            Some(span) => SynError::new(*span, &self.message),
            None => SynError::new(Span::call_site(), &self.message),
        }
    }

    /// Convert this error to a TokenStream with compile_error! macro
    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        self.to_syn_error().to_compile_error()
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MacroError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<SynError> for MacroError {
    fn from(error: SynError) -> Self {
        MacroError {
            message: error.to_string(),
            span: Some(error.span()),
            source: Some(Box::new(error)),
        }
    }
}

/// Helper function to create a multiple-error context
pub fn create_multiple_errors<T: fmt::Display>(message: T, errors: Vec<MacroError>, span: Span) -> MacroError {
    let mut combined_message = message.to_string();
    combined_message.push_str(":\n");
    
    for (i, error) in errors.iter().enumerate() {
        combined_message.push_str(&format!("    {}. {}\n", i + 1, error));
    }
    
    MacroError::new(combined_message, span)
} 