use five_core::{analysis::analyze_module, compile::{Compiled, Compiler}};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemMod};

#[proc_macro_attribute]
pub fn context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input item as an ItemMod
    let item_module: ItemMod = parse_macro_input!(item as ItemMod);

    // Make sure we have the module content
    let content = item_module.content.expect("Module must have a body");
    // Analyze the module
    let analyzed_module = analyze_module(&ItemMod {
        attrs: item_module.attrs,
        vis: item_module.vis,
        mod_token: item_module.mod_token,
        ident: item_module.ident,
        content: Some(content.clone()),
        semi: item_module.semi,
        unsafety: None,
    });

    // Compile and emit
    let compiled = analyzed_module.compile();
    let emitted = compiled.emit();

    // Convert to proc_macro::TokenStream
    proc_macro::TokenStream::from(emitted)
}