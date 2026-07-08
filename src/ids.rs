use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::codec;

const HASH_NAMESPACE: &[u8] = b"prism-dtl/hash/v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digest([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AccountId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EscrowId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TxId([u8; 32]);

impl Digest {
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn domain_hash(domain: &str, parts: &[&[u8]]) -> Self {
        Self(domain_hash(domain, parts))
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> crate::PrismResult<Self> {
        let encoded = serde_json::to_vec(value)
            .map_err(|err| crate::PrismError::Serialization(err.to_string()))?;
        Ok(Self::domain_hash(domain, &[encoded.as_slice()]))
    }
}

impl AccountId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_public_key(public_key: &[u8; 32]) -> Self {
        Self(domain_hash("account-id-v1", &[public_key]))
    }
}

impl AssetId {
    pub fn native() -> Self {
        Self(domain_hash("asset-id-v1", &[b"PUSD"]))
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl EscrowId {
    pub fn derive(depositor: AccountId, beneficiary: AccountId, nonce: u64, memo: Digest) -> Self {
        Self(domain_hash(
            "escrow-id-v1",
            &[
                depositor.bytes(),
                beneficiary.bytes(),
                &nonce.to_be_bytes(),
                memo.bytes(),
            ],
        ))
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TxId {
    pub const fn from_digest(digest: Digest) -> Self {
        Self(*digest.bytes())
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> crate::PrismResult<Self> {
        Digest::from_serializable(domain, value).map(Self::from_digest)
    }
}

fn domain_hash(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_NAMESPACE);
    hasher.update(&(domain.len() as u64).to_be_bytes());
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&(part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

macro_rules! hex_id {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                codec::serialize_hex(&self.0, serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Self(codec::deserialize_hex(deserializer)?))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", hex::encode(self.0))
            }
        }
    };
}

hex_id!(Digest);
hex_id!(AccountId);
hex_id!(AssetId);
hex_id!(EscrowId);
hex_id!(TxId);
