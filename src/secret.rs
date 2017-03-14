use std::collections::HashSet;

/// A Kubernetes secret.
///
/// If a set of these values is passed to a `Template`, all of the secret's data values will be
/// Base64 encoded after interpolation of parameters.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Secret {
    /// The name of the secret.
    pub name: String,
    /// The namespace of the secret.
    pub namespace: String,
}

/// A set of Kubernetes secrets.
pub type Secrets = HashSet<Secret>;
