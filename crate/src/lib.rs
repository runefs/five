use proc_macro::TokenStream;
use syn::token::Brace;
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, ItemMod};
use syn::visit_mut::VisitMut;
use quote::quote;

#[proc_macro_attribute]
pub fn context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input item as an ItemMod
    let input_module = parse_macro_input!(item as ItemMod);

    // Process the module
    let output = process_module(input_module);

    // Return the transformed module as a TokenStream
    TokenStream::from(output)
}

#[derive(Clone)]
struct Role {
    name: String,                       // Name of the trait
    default_impls: Vec<syn::TraitItem>, // Default implementations of methods
    super_trait: Option<syn::Path>,     // Path of the super trait, if any
}

use syn::{Expr, ExprMethodCall};

fn extract_lifetime_from_supertrait(supertrait: &syn::TypeParamBound) -> Option<syn::Lifetime> {
    if let syn::TypeParamBound::Trait(trait_bound) = supertrait {
        for segment in &trait_bound.path.segments {
            if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Lifetime(lifetime) = arg {
                        return Some(lifetime.clone());
                    }
                }
            }
        }
    }
    None
}

fn is_first_argument_mutable(sig: &syn::Signature) -> bool {
    if let Some(first_arg) = sig.inputs.first() {
        match first_arg {
            // Handle `self`, `&self`, `&mut self`
            syn::FnArg::Receiver(receiver) => receiver.reference.is_some() && receiver.mutability.is_some(),
            
            // Handle other arguments like `foo: i32`, `bar: &str`
            syn::FnArg::Typed(pat_type) => {
                if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                    type_ref.mutability.is_some()
                } else {
                    false
                }
            }
        }
    } else {
        false // No arguments means not mutable
    }
}

struct RoleAccessRewriter<'a> {
    pub roles: &'a [Role],
}

impl<'a> VisitMut for RoleAccessRewriter<'a> {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::MethodCall(method_call) = node {
            let method_call_clone = method_call.clone();

            self.rewrite_method_call(&method_call_clone, node);
        }

        syn::visit_mut::visit_expr_mut(self, node);
    }
}

impl<'a> RoleAccessRewriter<'a> {
    fn rewrite_method_call(
        &mut self,
        method_call: &ExprMethodCall,
        node: &mut Expr,
    ) {
        if let Expr::Field(field_expr) = &*method_call.receiver {
            if let Expr::Path(path_expr) = &*field_expr.base {
                if path_expr.path.is_ident("self") {
                    if let syn::Member::Named(field_name) = &field_expr.member {
                        // Match the field to the corresponding role
                        if let Some(role) = self.roles.iter().find(|role| {
                            role.name.to_lowercase()
                                == format!("{}role", field_name.to_string().to_lowercase())
                        }) {
                            let func_name = syn::Ident::new(
                                &format!("{}_{}", field_name.to_string().to_lowercase(), method_call.method),
                                method_call.method.span(),
                            );
        
                            let mut new_args = syn::punctuated::Punctuated::new();
        
                            // Create a reference to the receiver
                            let rewritten_receiver = Expr::Reference(syn::ExprReference {
                                attrs: vec![],
                                and_token: Default::default(),
                                mutability: if is_first_argument_mutable(&role.default_impls.iter()
                                    .find_map(|item| {
                                        if let syn::TraitItem::Fn(fn_item) = item {
                                            if fn_item.sig.ident == method_call.method {
                                                Some(&fn_item.sig)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap()) {
                                    Some(syn::token::Mut::default())
                                } else {
                                    None
                                },
                                expr: Box::new(Expr::Field(field_expr.clone())),
                            });
        
                            new_args.push(rewritten_receiver);
                            new_args.extend(method_call.args.clone());
        
                            // Replace the method call with a function call
                            *node = Expr::Call(syn::ExprCall {
                                attrs: vec![],
                                func: Box::new(Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path::from(func_name),
                                })),
                                paren_token: Default::default(),
                                args: new_args,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn remove_role_traits(
    content: &mut Option<(Brace, Vec<syn::Item>)>
) -> Vec<Role> {
    let mut roles = Vec::new();

    if let Some((_, ref mut items)) = content {
        // Collect role traits and remove them from `items`
        items.retain(|item| {
            if let syn::Item::Trait(trait_item) = item {
                if trait_item.ident.to_string().ends_with("Role") {
                    // Extract super trait (if any)
                    let super_trait = trait_item.supertraits.iter().find_map(|bound| {
                        if let syn::TypeParamBound::Trait(trait_bound) = bound {
                            Some(trait_bound.path.clone())
                        } else {
                            None
                        }
                    });

                    // Extract default implementations
                    let default_impls = trait_item
                        .items
                        .iter()
                        .cloned()
                        .filter_map(|trait_item| {
                            if let syn::TraitItem::Fn(method) = trait_item {
                                Some(syn::TraitItem::Fn(method))
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Create and collect the Role
                    roles.push(Role {
                        name: trait_item.ident.to_string(),
                        super_trait,
                        default_impls,
                    });

                    return false; // Remove this item
                }
            }
            true // Retain non-Role items
        });
    }

    roles
}


fn rewrite_role_accesses(context_impl: &mut syn::ItemImpl, roles: &[Role]) {
    let mut rewriter = RoleAccessRewriter{roles};
    rewriter.visit_item_impl_mut(context_impl);
}

fn rewrite_sig_of_free_function(
    sig: &mut syn::Signature,
    super_trait: Option<syn::Path>,
    role_prefix: &str,
) {
    // Prepare the generic type `T`
    let generic_type = syn::Ident::new("T", proc_macro2::Span::call_site());

    // Prefix the function name with the `role_prefix` in lower_snake_case
    let prefixed_fn_name = format!(
        "{}_{}",
        role_prefix.to_lowercase(),
        sig.ident.to_string()
    );
    sig.ident = syn::Ident::new(&prefixed_fn_name, sig.ident.span());

    // Replace `&self` or `&mut self` with `this: &T` or `this: &mut T`
    if let Some(first_arg) = sig.inputs.first_mut() {
        if let syn::FnArg::Receiver(receiver) = first_arg {
            // Extract mutability from the receiver (`&self` or `&mut self`)
            let mutability = receiver.mutability;

            // Replace the receiver with `this: &T` or `this: &mut T`
            *first_arg = syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None, // Ensure `mut` is not redundantly applied here
                    ident: syn::Ident::new("this", proc_macro2::Span::call_site()),
                    subpat: None,
                })),
                colon_token: Default::default(),
                ty: Box::new(syn::Type::Reference(syn::TypeReference {
                    and_token: Default::default(),
                    lifetime: None,
                    mutability, // Apply mutability to the reference type
                    elem: Box::new(syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![syn::PathSegment {
                                ident: generic_type.clone(),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    })),
                })),
            });
        }
    }

    // Add the generic type `T`
    sig.generics.params.push(syn::GenericParam::Type(syn::TypeParam {
        attrs: vec![],
        ident: generic_type.clone(),
        colon_token: None,
        bounds: Punctuated::new(),
        eq_token: None,
        default: None,
    }));

    // Add a where clause if a super trait is provided
    if let Some(super_trait_path) = super_trait {
        if sig.generics.where_clause.is_none() {
            sig.generics.where_clause = Some(syn::WhereClause {
                where_token: Default::default(),
                predicates: Punctuated::new(),
            });
        }

        if let Some(where_clause) = &mut sig.generics.where_clause {
            where_clause.predicates.push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: vec![syn::PathSegment {
                            ident: generic_type.clone(),
                            arguments: syn::PathArguments::None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                }),
                colon_token: Default::default(),
                bounds: vec![syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: super_trait_path.clone(),
                })]
                .into_iter()
                .collect(),
            }));
        }
    }
}

fn rewrite_body_of_free_function(block: &mut syn::Block) {
    use syn::visit_mut::{self, VisitMut};

    struct SelfReplacer {
        this_ident: syn::Ident,
    }

    impl VisitMut for SelfReplacer {
        fn visit_expr_path_mut(&mut self, expr_path: &mut syn::ExprPath) {
            if expr_path.path.is_ident("self") {
                expr_path.path.segments.clear();
                expr_path.path.segments.push(syn::PathSegment {
                    ident: self.this_ident.clone(),
                    arguments: syn::PathArguments::None,
                });
            }
            visit_mut::visit_expr_path_mut(self, expr_path);
        }
    }

    let mut replacer = SelfReplacer {
        this_ident: syn::Ident::new("this", proc_macro2::Span::call_site()),
    };

    replacer.visit_block_mut(block);
}

fn validate_context_have_only_role_members(context_struct: &syn::ItemStruct) {
    if let syn::Visibility::Public(_) = context_struct.vis {
        panic!("`Context` must be private.");
    }

    for field in context_struct.fields.iter() {
        if let Some(ident) = &field.ident {
            let ty = &field.ty;

            match ty {
                // Check for types ending with "Role"
                syn::Type::Path(type_path) => {
                    let last_segment = type_path.path.segments.last().unwrap();
                    let type_name = last_segment.ident.to_string();
                    if !type_name.ends_with("Role") && !is_primitive_type(&type_name) {
                        panic!(
                            "Field `{}` in `Context` must be a primitive type or a struct ending in `Role`.",
                            ident
                        );
                    }
                }
                // Disallow other types
                _ => panic!(
                    "Field `{}` in `Context` must be a primitive type or a struct ending in `Role`.",
                    ident
                ),
            }
        }
    }
}

/// Helper function to check if a type name represents a primitive
fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "i32" | "i64" | "f32" | "f64" | "bool" | "char" | "u8" | "u16" | "u32" | "u64" | "usize"
    )
}

fn create_new_context(
    original: syn::ItemStruct,
    roles: &[Role],
) -> syn::ItemStruct {
    let mut new_struct = original.clone();
    new_struct.generics = syn::Generics::default(); // Reset generics for the new struct

    let mut generics = Vec::new();
    let mut where_clause = syn::WhereClause {
        where_token: Default::default(),
        predicates: Punctuated::new(),
    };

    // Map to track reused generic type arguments
    let mut type_to_generic: std::collections::HashMap<String, syn::Ident> = std::collections::HashMap::new();

    // Track lifetimes used in supertraits
    let mut lifetime_set: std::collections::HashSet<syn::Lifetime> = std::collections::HashSet::new();

    if let syn::Fields::Named(ref mut fields_named) = new_struct.fields {
        // Create a temporary vector to hold new fields
        let mut new_named_fields = Punctuated::new();

        for field in fields_named.named.iter() {
            if let syn::Type::Path(type_path) = &field.ty {
                let last_segment = type_path.path.segments.last().unwrap();
                let type_name = last_segment.ident.to_string();

                // Check if the field's type ends with "Role"
                if type_name.ends_with("Role") {
                    // Reuse an existing generic argument if the type is already processed
                    let generic_ident = if let Some(existing_generic) = type_to_generic.get(&type_name) {
                        existing_generic.clone()
                    } else {
                        let new_generic = syn::Ident::new(&format!("T{}", type_name), type_path.span());
                        type_to_generic.insert(type_name.clone(), new_generic.clone());

                        // Add the generic parameter
                        generics.push(syn::GenericParam::Type(syn::TypeParam {
                            attrs: vec![],
                            ident: new_generic.clone(),
                            colon_token: None,
                            bounds: Punctuated::new(),
                            eq_token: None,
                            default: None,
                        }));

                        // Add a bound if the role has a super trait
                        if let Some(role) = roles.iter().find(|r| r.name == type_name) {
                            if let Some(super_trait) = &role.super_trait {
                                // Extract any lifetime from the supertrait
                                if let Some(lifetime) = extract_lifetime_from_supertrait(&syn::TypeParamBound::Trait(
                                    syn::TraitBound {
                                        path: super_trait.clone(),
                                        paren_token: None,
                                        modifier: syn::TraitBoundModifier::None,
                                        lifetimes: None,
                                    },
                                )) {
                                    lifetime_set.insert(lifetime.clone());
                                }

                                where_clause.predicates.push(syn::WherePredicate::Type(
                                    syn::PredicateType {
                                        lifetimes: None,
                                        bounded_ty: syn::Type::Path(syn::TypePath {
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: vec![syn::PathSegment {
                                                    ident: new_generic.clone(),
                                                    arguments: syn::PathArguments::None,
                                                }]
                                                .into_iter()
                                                .collect(),
                                            },
                                        }),
                                        colon_token: Default::default(),
                                        bounds: vec![syn::TypeParamBound::Trait(syn::TraitBound {
                                            paren_token: None,
                                            modifier: syn::TraitBoundModifier::None,
                                            lifetimes: None,
                                            path: super_trait.clone(),
                                        })]
                                        .into_iter()
                                        .collect(),
                                    },
                                ));
                            }
                        }
                        new_generic
                    };

                    // Replace the field type with the reused or new generic type
                    let mut new_field = field.clone();
                    new_field.ty = syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![syn::PathSegment {
                                ident: generic_ident.clone(),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    });
                    new_named_fields.push(new_field);
                } else {
                    // Retain fields with primitive types
                    new_named_fields.push(field.clone());
                }
            }
        }

        // Replace the original named fields with the new ones
        fields_named.named = new_named_fields;
    }

    // Add collected lifetimes to generics
    for lifetime in lifetime_set {
        generics.push(syn::GenericParam::Lifetime(syn::LifetimeParam {
            attrs: vec![],
            lifetime,
            colon_token: None,
            bounds: Punctuated::new(),
        }));
    }

    // Update the new struct with transformed fields and generics
    new_struct.generics.params.extend(generics);
    if !where_clause.predicates.is_empty() {
        new_struct.generics.where_clause = Some(where_clause);
    }

    new_struct
}

fn recreate_context_impl(
    original_impl: syn::ItemImpl,
    new_struct: &syn::ItemStruct,
) -> syn::ItemImpl {
    let mut new_impl = original_impl.clone();

    // Handle `for<'a>` quantifiers in the original implementation
    if let Some(for_lifetimes) = &original_impl.generics.lt_token {
        new_impl.generics.lt_token = Some(for_lifetimes.clone());
    }

    // Handle generic parameters and lifetimes
    if let syn::Type::Path(type_path) = &mut *new_impl.self_ty {
        let last_segment = type_path.path.segments.last_mut().unwrap();

        last_segment.arguments = syn::PathArguments::AngleBracketed(
            syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Default::default(),
                args: new_struct
                    .generics
                    .params
                    .iter()
                    .map(|param| match param {
                        syn::GenericParam::Lifetime(lifetime) => {
                            syn::GenericArgument::Lifetime(lifetime.lifetime.clone())
                        }
                        syn::GenericParam::Type(type_param) => {
                            syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: syn::Path {
                                    leading_colon: None,
                                    segments: vec![syn::PathSegment {
                                        ident: type_param.ident.clone(),
                                        arguments: syn::PathArguments::None,
                                    }]
                                    .into_iter()
                                    .collect(),
                                },
                            }))
                        }
                        _ => panic!("Unexpected generic parameter type"),
                    })
                    .collect(),
                gt_token: Default::default(),
            },
        );
    }

    new_impl
}


fn to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|c| c.to_ascii_uppercase())
                .into_iter()
                .chain(chars)
                .collect::<String>()
        })
        .collect()
}

fn process_context(
    context_struct: syn::ItemStruct,
    context_impl: Option<syn::ItemImpl>,
    roles: &[Role],
) -> (syn::ItemStruct, Option<syn::ItemImpl>) {
    // Validate and recreate the `Context` struct
    validate_context_have_only_role_members(&context_struct);
    let new_struct = create_new_context(context_struct, roles);

    // Recreate and rewrite the `impl` block
    let new_impl = context_impl.map(|mut impl_block| {
        rewrite_role_accesses(&mut impl_block, roles);
        recreate_context_impl(impl_block, &new_struct)
    });

    (new_struct, new_impl)
}

fn generate_context_trait(context_impl: &syn::ItemImpl, module_name: &syn::Ident) -> syn::ItemTrait {
    let trait_name_str = format!("{}_ctx", module_name);
    let trait_name = syn::Ident::new(&to_pascal_case(&trait_name_str), module_name.span());

    let generics = &context_impl.generics;
    let (_impl_generics,_, where_clause) = generics.split_for_impl();

    let methods = context_impl
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                let sig = &method.sig;
                Some(quote! { #sig; })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    syn::parse_quote! {
        pub trait #trait_name #where_clause {
            #(#methods)*
        }
    }
}

fn ensure_impl_trait(
    context_impl: &mut syn::ItemImpl,
    trait_name: &syn::Ident,
    context_struct: &syn::ItemStruct,
) -> syn::ItemImpl {
    let struct_name = &context_struct.ident;

    // Extract generics and bounds from the context struct
    let generics = &context_struct.generics;
    let (_impl_generics, _, _where_clause) = generics.split_for_impl();

    // Map the generics to `GenericArgument` for the `PathSegment.arguments`
    let generic_arguments: syn::punctuated::Punctuated<_, syn::token::Comma> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => Some(syn::GenericArgument::Type(syn::Type::Path(
                syn::TypePath {
                    qself: None,
                    path: syn::Path::from(type_param.ident.clone()),
                },
            ))),
            syn::GenericParam::Lifetime(lifetime_param) => Some(syn::GenericArgument::Lifetime(
                lifetime_param.lifetime.clone(),
            )),
            _ => None,
        })
        .collect();

    // Add the trait implementation to the `impl` block
    let mut unified_impl = context_impl.clone();
    unified_impl.trait_ = Some((
        None, // No `for<>` higher-ranked lifetimes
        syn::Path::from(trait_name.clone()), // Path to the trait
        Default::default(), // No `as` tokens
    ));

    // Apply generics and bounds to the `impl` block
    unified_impl.generics = generics.clone();
    unified_impl.self_ty = Box::new(syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: vec![syn::PathSegment {
                ident: struct_name.clone(),
                arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Default::default(),
                    args: generic_arguments, // Corrected to use mapped generic arguments
                    gt_token: Default::default(),
                }),
            }]
            .into_iter()
            .collect(),
        },
    }));

    // Return the modified `impl` block
    unified_impl
}

fn generate_bind_function(
    context_struct: &syn::ItemStruct,
    module_name: &syn::Ident,
) -> syn::ItemFn {
    let struct_name = &context_struct.ident;
    let trait_name_str = format!("{}_ctx", module_name);
    let trait_name = syn::Ident::new(&to_pascal_case(&trait_name_str), module_name.span());

    // Extract the generics from the `Context` struct
    let generics = &context_struct.generics;
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // Collect arguments for the `bind` function
    let args = if let syn::Fields::Named(fields) = &context_struct.fields {
        fields.named.iter().map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;
            quote! { #field_name: #field_type }
        })
    } else {
        panic!("Context struct must have named fields.");
    };

    // Collect field initializers for constructing the `Context` instance
    let field_initializers = if let syn::Fields::Named(fields) = &context_struct.fields {
        fields.named.iter().map(|field| {
            let field_name = &field.ident;
            quote! { #field_name }
        })
    } else {
        panic!("Context struct must have named fields.");
    };

    // Generate the `bind` function with generics
    syn::parse_quote! {
        pub fn bind #impl_generics(#(#args),*) -> impl #trait_name #where_clause {
            #struct_name {
                #(#field_initializers),*
            }
        }
    }
}

fn process_module(mut module: ItemMod) -> proc_macro2::TokenStream {
    let content = &mut module.content;
    let roles = remove_role_traits(content);

    // Generate tokens for default implementations
    let default_impls_tokens = 
        roles.clone().into_iter().flat_map(|role| {
            let role_prefix = role.name.strip_suffix("Role").unwrap_or(&role.name); // Get `XXX` from `XXXRole`
            role.clone().default_impls.into_iter().filter_map(move |trait_item| {
                if let syn::TraitItem::Fn(method) = trait_item {
                    if let Some(block) = method.default {
                        let mut sig = method.sig;
                        
                        rewrite_sig_of_free_function(&mut sig, role.super_trait.clone(), role_prefix);
                        
                        let mut body = block.clone();
                        rewrite_body_of_free_function(&mut body);
                        Some(quote! {
                            #sig #body
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect::<Vec<_>>()
        });

    let mut context_struct: Option<syn::ItemStruct> = None;
    let mut context_impl: Option<syn::ItemImpl> = None;
    let mut new_items = Vec::new();

    if let Some((_, items)) = content {
        // Remove `Context` struct and `impl` block
        items.retain(|item| match item {
            syn::Item::Struct(context) if context.ident == "Context" => {
                context_struct = Some(context.clone());
                false // Remove the `Context` struct
            }
            syn::Item::Impl(impl_block) => {
                // Check if the `impl` block is for the `Context` struct
                if let syn::Type::Path(type_path) = &*impl_block.self_ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Context" {
                            context_impl = Some(impl_block.clone());
                            return false; // Remove the `impl` block for `Context`
                        }
                    }
                }
                true
            }
            _ => true,
        });

        if let Some(ctx) = context_struct {
            if let Some(ctx_impl) = context_impl.take() {
                
                let (new_context_struct, new_context_impl) =
                    process_context(ctx, Some(ctx_impl), &roles);

                // Generate the trait
                let module_name = &module.ident;
                let mut ctx_impl = new_context_impl.unwrap();
                let context_trait = generate_context_trait(&ctx_impl, module_name);

                // Ensure the `impl` block implements the trait
                ensure_impl_trait(&mut ctx_impl, &context_trait.ident, &new_context_struct);

                // Generate the `bind` function
                let bind_function = generate_bind_function(&new_context_struct, module_name);

                // Add the transformed `Context` struct, `impl` block, and new elements
                items.push(syn::Item::Struct(new_context_struct));
                items.push(syn::Item::Impl(ctx_impl));
                items.push(syn::Item::Trait(context_trait));
                items.push(syn::Item::Fn(bind_function));
            } else {
                panic!("No `impl` block found for the `Context` struct.");
            }
        } else {
            panic!("No `Context` struct found in the module.");
        }
    
        // Add all default implementations as functions to the module
        for tokens in default_impls_tokens {
            if let Ok(item) = syn::parse2::<syn::Item>(tokens) {
                new_items.push(item);
            }
        }
    
        // Append new items to the module's content
        items.extend(new_items);
    }

    // Return the mutated module
    quote! {
        #module
    }
}
