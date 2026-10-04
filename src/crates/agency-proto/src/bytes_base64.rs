//! Fixed RFC 4648 padded encoding for arbitrary bytes in canonical JSON.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&STANDARD.encode(bytes))
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    let encoded = String::deserialize(deserializer)?;
    STANDARD.decode(encoded).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Bytes {
        #[serde(with = "super")]
        bytes: Vec<u8>,
    }
    #[test]
    fn arbitrary_bytes_roundtrip_in_canonical_padded_string() {
        let bytes: Vec<u8> = (0..=255).collect();
        let json = crate::canonical(&Bytes {
            bytes: bytes.clone(),
        })
        .unwrap();
        assert!(json.len() < 360);
        assert_eq!(serde_json::from_slice::<Bytes>(&json).unwrap().bytes, bytes);
        for invalid in [
            r#"{"bytes":[1,2]}"#,
            r#"{"bytes":"YQ"}"#,
            r#"{"bytes":"YR=="}"#,
        ] {
            assert!(serde_json::from_str::<Bytes>(invalid).is_err());
        }
    }
}
