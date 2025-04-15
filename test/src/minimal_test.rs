// Minimal test case to demonstrate the "non-constant value in a constant" error

// Trait definitions
pub trait TraitA {
    fn foo(&self) -> i32;
}

pub trait TraitB {
    fn bar(&self) -> bool;
}

// Using dyn trait objects directly in struct fields
struct ContextWithTraits {
    a: Box<dyn TraitA>,  // This works at runtime but will have problems in compile-time contexts
    b: Box<dyn TraitB>   // This works at runtime but will have problems in compile-time contexts
}

// Using generics instead - this is the preferred solution
struct ContextWithGenerics<A: TraitA, B: TraitB> {
    a: A,
    b: B
}

// Implementations to test
struct ImplA;
impl TraitA for ImplA {
    fn foo(&self) -> i32 { 42 }
}

struct ImplB;
impl TraitB for ImplB {
    fn bar(&self) -> bool { true }
}

// This demonstrates the problem in the macro context:
// When a macro tries to define a struct with trait objects in a constant context,
// you'd get an error like:
//
// const _ISSUE: ContextWithTraits = ContextWithTraits {
//     a: Box::new(ImplA), // ERROR: attempt to use a non-constant value in a constant
//     b: Box::new(ImplB)  // ERROR: attempt to use a non-constant value in a constant
// };

// But this works fine with generics:
const _WORKS: ContextWithGenerics<ImplA, ImplB> = ContextWithGenerics {
    a: ImplA,
    b: ImplB
};

fn main() {
    // This works fine at runtime
    let ctx1 = ContextWithTraits { 
        a: Box::new(ImplA), 
        b: Box::new(ImplB) 
    };
    
    // This also works
    let ctx2 = ContextWithGenerics { a: ImplA, b: ImplB };
    
    assert_eq!(ctx1.a.foo(), 42);
    assert!(ctx1.b.bar());
    assert_eq!(ctx2.a.foo(), 42);
    assert!(ctx2.b.bar());
} 