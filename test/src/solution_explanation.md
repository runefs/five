# Fixing "attempt to use a non-constant value in a constant" Error in Rust

## Problem

When using trait objects directly in struct fields within a macro-generated context, Rust produces an error:
"attempt to use a non-constant value in a constant".

This happens because:

1. The `five::context` macro is likely generating code that treats the `Context` struct in a constant context
2. Trait objects (like `TraitA` or `SerialiserRole`) require runtime allocation via `Box<dyn TraitA>` which cannot be done in a constant context
3. Box allocations are not allowed in constant expressions since they require runtime heap allocation

## Solution

The solution is to use generic type parameters instead of trait objects directly:

```rust
// Original (problematic)
struct Context {
    serialiser: SerialiserRole,
    encrypter: EncrypterRole,
    store: StoreRole
}

// Fixed version with generics
struct Context<S: SerialiserRole, E: EncrypterRole, St: StoreRole> {
    serialiser: S,
    encrypter: E,
    store: St
}

#[async_trait::async_trait]
impl<S, E, St> Context<S, E, St> 
where 
    S: SerialiserRole + Sync + Send,
    E: EncrypterRole + Sync + Send,
    St: StoreRole + Sync + Send
{
    // Method implementations
}
```

## Benefits

1. **Compile-time Safety**: Generic parameters are resolved at compile time, with no runtime allocations needed
2. **Constant Context Compatible**: Generic structs can be used in constant contexts when the type parameters are known
3. **Performance**: Generic implementations can often be faster due to monomorphization, where the compiler generates specialized code for each type
4. **Same Functionality**: This approach maintains the same functionality as the original trait object approach

## Limitations to Consider

1. **Code Size**: Generics can increase binary size due to monomorphization
2. **Flexibility**: You lose the ability to store heterogeneous implementations in containers (but this wasn't being used here anyway)

## Root Cause in the Macro

The issue occurs because the procedural macro likely uses the Context struct in a compile-time context when generating code. In Rust, a trait object like `SerialiserRole` represents a value whose type isn't known at compile time, which makes it incompatible with compile-time constant evaluation.

By using generics, the Context struct becomes usable in both runtime and constant contexts, which solves the problem while maintaining the same functionality. 