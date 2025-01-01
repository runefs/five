use syn::{Ident,Type};

#[derive(Clone)]
pub struct PropertyInfo {
    name: Ident,
    ty: Type,
}
impl PropertyInfo {
    pub fn new(name: Ident, ty: Type) -> Self {
        PropertyInfo { name, ty }
    }

    pub fn get_name(&self) -> Ident {
        self.name.clone()
    }

    pub fn get_ty(&self) -> Type {
        self.ty.clone()
    }
}