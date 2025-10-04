// erbium/crypto/src/lib.rs

//! This crate defines the core cryptographic primitives for the Erbium blockchain,
//! such as key generation, signing, and verification.

// --- CORRECTED IMPORTS ---
// We import and publicly re-export all necessary types in a single, clean statement.
pub use ed25519_dalek::{Signature, VerifyingKey, Signer, Verifier, SigningKey};
use rand::rngs::OsRng;
// -------------------------


/// Represents a cryptographic keypair (private and public key).
#[derive(Debug)] // Clone is not needed here as we don't copy the Keypair itself
pub struct Keypair {
    /// The private part of the keypair, used for signing.
    pub private: SigningKey,
    /// The public part of the keypair, used for verification.
    pub public: VerifyingKey,
}

impl Keypair {
    /// Generates a new, random keypair.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let private = SigningKey::generate(&mut csprng);
        let public = private.verifying_key();
        
        Keypair { private, public }
    }

    /// Signs a given message (as bytes) with the private key.
    /// Returns a cryptographic signature.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.private.sign(message)
    }

    /// Verifies a signature against a message with the public key.
    /// Returns `true` if the signature is valid, `false` otherwise.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.public.verify(message, signature).is_ok()
    }
}