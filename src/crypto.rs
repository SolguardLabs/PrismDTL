use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    AccountId, PrismError, PrismResult, codec,
    error::PrismError::Serialization as SerializationError,
};

const SIGNING_NAMESPACE: &[u8] = b"prism-dtl/signing/v1";

#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PublicKeyBytes([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignatureBytes([u8; 64]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub account: AccountId,
    pub public_key: PublicKeyBytes,
}

impl KeyPair {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        Self {
            signing_key: SigningKey::generate(&mut rng),
        }
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_identity(&self) -> PublicIdentity {
        let public_key = PublicKeyBytes(*self.signing_key.verifying_key().as_bytes());
        PublicIdentity {
            account: AccountId::from_public_key(public_key.bytes()),
            public_key,
        }
    }

    pub fn sign<T: Serialize>(&self, domain: &str, value: &T) -> PrismResult<SignatureBytes> {
        let payload = signing_payload(domain, value)?;
        Ok(SignatureBytes(self.signing_key.sign(&payload).to_bytes()))
    }
}

impl PublicKeyBytes {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    fn verifying_key(self) -> PrismResult<VerifyingKey> {
        VerifyingKey::from_bytes(&self.0).map_err(|_| PrismError::InvalidPublicKey)
    }
}

impl SignatureBytes {
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    fn signature(self) -> Signature {
        Signature::from_bytes(&self.0)
    }
}

impl PublicIdentity {
    pub fn verify_consistency(self) -> PrismResult<()> {
        let expected = AccountId::from_public_key(self.public_key.bytes());
        if expected != self.account {
            return Err(PrismError::IdentityMismatch(self.account));
        }
        Ok(())
    }
}

pub fn verify<T: Serialize>(
    identity: PublicIdentity,
    signature: SignatureBytes,
    domain: &str,
    value: &T,
) -> PrismResult<()> {
    identity.verify_consistency()?;
    let payload = signing_payload(domain, value)?;
    identity
        .public_key
        .verifying_key()?
        .verify(&payload, &signature.signature())
        .map_err(|_| PrismError::InvalidSignature)
}

fn signing_payload<T: Serialize>(domain: &str, value: &T) -> PrismResult<Vec<u8>> {
    let encoded = serde_json::to_vec(value).map_err(|err| SerializationError(err.to_string()))?;
    let mut payload = Vec::with_capacity(SIGNING_NAMESPACE.len() + domain.len() + encoded.len());
    payload.extend_from_slice(SIGNING_NAMESPACE);
    payload.extend_from_slice(&(domain.len() as u64).to_be_bytes());
    payload.extend_from_slice(domain.as_bytes());
    payload.extend_from_slice(&(encoded.len() as u64).to_be_bytes());
    payload.extend_from_slice(&encoded);
    Ok(payload)
}

impl Serialize for PublicKeyBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        codec::serialize_hex(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKeyBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(codec::deserialize_hex(deserializer)?))
    }
}

impl Serialize for SignatureBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        codec::serialize_hex(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for SignatureBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(codec::deserialize_hex(deserializer)?))
    }
}
