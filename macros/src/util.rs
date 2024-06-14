use syn::{GenericArgument, Path, PathArguments, PathSegment};

/// Check if the supplied type matches at least one of the provided signatures.
///
/// Note that this method is not guaranteed to work on every possible input
/// since we don't have access to type information in the AST representation.
///
/// Adapted from https://stackoverflow.com/a/56264023
pub fn matches_signature(ty: &syn::Type, signatures: &[&str]) -> bool {
    if let Some(path) = extract_type_path(ty) {
        let signature =
            path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
        signatures.contains(&signature.as_ref())
    } else {
        false
    }
}

/// Strips leading r# from the ident. Used to convert idents to string literals.
pub fn strip_leading_rawlit(s: &str) -> String {
    if s.starts_with("r#") {
        s.chars().skip(2).collect()
    } else {
        s.to_owned()
    }
}

/// Check if the supplied type matches the [Option] signature.
pub fn matches_option_signature(ty: &syn::Type) -> bool {
    matches_signature(ty, &["Option", "std::option::Option", "core::option::Option"])
}

/// Check if the supplied type matches the [Vec] signature.
pub fn matches_vec_signature(ty: &syn::Type) -> bool {
    matches_signature(ty, &["Vec", "std::vec::Vec"])
}

/// Check if the supplied type matches the [Vec] signature.
pub fn matches_hashmap_signature(ty: &syn::Type) -> bool {
    matches_signature(ty, &["HashMap", "std::collections::HashMap"])
}

pub fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

pub fn extract_value_type_from_hashmap(ty: &syn::Type) -> Option<&syn::Type> {
    fn extract_hashmap_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
            acc.push_str(&v.ident.to_string());
            acc.push('|');
            acc
        });
        vec!["HashMap|", "std|collections|HashMap|"]
            .into_iter()
            .find(|s| idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_hashmap_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.last(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}
