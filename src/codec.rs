use serde::{Deserialize, Deserializer, Serializer, de};

pub(crate) fn serialize_hex<const N: usize, S>(
    bytes: &[u8; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(bytes))
}

pub(crate) fn deserialize_hex<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let bytes = hex::decode(encoded).map_err(de::Error::custom)?;
    if bytes.len() != N {
        return Err(de::Error::custom(format!(
            "invalid byte length: expected {N}, received {}",
            bytes.len()
        )));
    }

    let mut out = [0u8; N];
    out.copy_from_slice(&bytes);
    Ok(out)
}
