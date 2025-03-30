use proc_macro2::TokenStream;
pub trait Compiler<T> {
    type Output: Compiled<T>;

    fn compile(&self) -> Self::Output;
}

pub trait Compiled<T> {
    fn emit(&self) -> TokenStream;
}
