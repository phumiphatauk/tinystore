//! Authentication and authorization for TinyStore

pub mod error;
pub mod signature_v4;
pub mod credentials;
pub mod middleware;

pub use error::{AuthError, AuthResult};
pub use signature_v4::SignatureV4;
pub use credentials::{Credentials, CredentialStore};
pub use middleware::AuthLayer;
