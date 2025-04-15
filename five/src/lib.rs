mod analysis;
mod compile;

use crate::analysis::analyze_module;
use crate::compile::{Compiled, Compiler};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{ItemMod, Error};
use proc_macro_error::{abort, proc_macro_error};

/// Procedural macro for creating a context module with roles and contracts.
/// This macro provides better error reporting that points to the specific location
/// where issues occur within the module, rather than just the macro invocation site.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Store original item for error recovery
    let original_item = item.clone();
    
    // Parse the input item as an ItemMod
    let item_module: ItemMod = match syn::parse(item.clone()) {
        Ok(module) => module,
        Err(err) => {
            let error_msg = format!("Failed to parse module: {}", err);
            eprintln!("Error in five::context macro: {}", error_msg);
            abort!(Span::call_site(), error_msg);
        }
    };

    // Debug output - useful for understanding macro input
    if std::env::var("FIVE_DEBUG").is_ok() {
        eprintln!("Processing module `{}`", item_module.ident);
    }

    // Make sure we have the module content
    let content = match &item_module.content {
        Some(content) => content.clone(),
        None => {
            let error_msg = "Module must have a body (missing braces)";
            eprintln!("Error in five::context macro: {}", error_msg);
            abort!(item_module.ident.span(), error_msg);
        }
    };
    
    // Analyze the module items before processing
    if std::env::var("FIVE_DEBUG").is_ok() {
        for item in &content.1 {
            match item {
                syn::Item::Struct(item_struct) => {
                    eprintln!("Found struct: {}", item_struct.ident);
                    if item_struct.ident == "Context" {
                        eprintln!("Context struct fields:");
                        for field in item_struct.fields.iter() {
                            if let Some(ident) = &field.ident {
                                eprintln!("  {} : {}", ident, field.ty.to_token_stream());
                            }
                        }
                    }
                },
                syn::Item::Trait(item_trait) => {
                    eprintln!("Found trait: {}", item_trait.ident);
                },
                syn::Item::Impl(item_impl) => {
                    if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            eprintln!("Found impl for: {}", segment.ident);
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    // Analyze the module
    let analyzed_module = match analyze_module_with_error_reporting(&ItemMod {
        attrs: item_module.attrs,
        vis: item_module.vis,
        mod_token: item_module.mod_token,
        ident: item_module.ident.clone(),
        content: Some(content),
        semi: item_module.semi,
        unsafety: None,
    }) {
        Ok(module) => module,
        Err(err) => {
            eprintln!("Analysis error in five::context macro: {}", err);
            // Return the original code instead of aborting to aid debugging
            if std::env::var("FIVE_FALLBACK").is_ok() {
                eprintln!("Using original code as fallback");
                return original_item;
            }
            return TokenStream::from(err.to_compile_error());
        }
    };

    // Compile and emit
    let compiled = match compile_module_with_error_reporting(analyzed_module) {
        Ok(compiled) => compiled,
        Err(err) => {
            eprintln!("Compilation error in five::context macro: {}", err);
            // Return the original code instead of aborting to aid debugging
            if std::env::var("FIVE_FALLBACK").is_ok() {
                eprintln!("Using original code as fallback");
                return original_item;
            }
            return TokenStream::from(err.to_compile_error());
        }
    };
    
    let emitted = compiled.emit();
    
    // Debug output - very useful for diagnosing issues
    if std::env::var("FIVE_DEBUG").is_ok() {
        eprintln!("Generated code for module `{}`:", item_module.ident);
        eprintln!("{}", emitted);
    }

    // Convert to proc_macro::TokenStream
    proc_macro::TokenStream::from(emitted)
}

// Helper function to analyze a module with better error reporting
fn analyze_module_with_error_reporting(module: &ItemMod) -> Result<analysis::ModuleInfo, Error> {
    // Count the Context structs
    let mut context_count = 0;
    if let Some((_, items)) = &module.content {
        for item in items {
            if let syn::Item::Struct(item_struct) = item {
                if item_struct.ident == "Context" {
                    context_count += 1;
                }
            }
        }
    }
    
    // Validate Context struct count
    if context_count == 0 {
        return Err(Error::new(
            module.ident.span(),
            "Missing Context struct - each module must define exactly one Context struct"
        ));
    } else if context_count > 1 {
        return Err(Error::new(
            module.ident.span(),
            format!("Found {} Context structs - each module must define exactly one Context struct", context_count)
        ));
    }
    
    // Proceed with analysis if validation passes
    Ok(analyze_module(module))
}

// Helper function to compile a module with better error reporting
fn compile_module_with_error_reporting(module: analysis::ModuleInfo) -> Result<compile::module::CompiledModule, Error> {
    // Check if the Context struct has properties
    if module.context.properties.is_empty() {
        return Err(Error::new(
            module.context.name.span(),
            "Context struct must have at least one property"
        ));
    }
    
    // Check if we have any impl blocks for the Context struct
    if module.context.impl_blocks.is_empty() {
        return Err(Error::new(
            module.context.name.span(),
            "Context struct must have at least one impl block"
        ));
    }
    
    // Proceed with compilation if validation passes
    Ok(module.compile())
}
