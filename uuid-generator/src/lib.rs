use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::{ContextV7, Timestamp, Uuid};
use uuid_simd::UuidExt;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct UUID(pub Uuid);

impl Default for UUID {
    fn default() -> Self {
        Self::new_v7()
    }
}

impl UUID {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn new_v7() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn new_v7_with_timestamp(ts: Timestamp) -> Self {
        Self(Uuid::new_v7(ts))
    }

    pub fn simple(&self) -> String {
        self.0.format_simple().to_string()
    }

    pub fn hyphenated(&self) -> String {
        self.0.format_hyphenated().to_string()
    }
}

impl FromStr for UUID {
    type Err = uuid_simd::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UUID(Uuid::parse(s.as_bytes())?))
    }
}

pub fn batch_uuid_v4(size: usize) -> Vec<UUID> {
    (0..size).map(|_| UUID::new_v4()).collect()
}

pub fn batch_uuid_v7(size: usize) -> Vec<UUID> {
    let context = ContextV7::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let secs = now.as_secs();
    let nanos = now.subsec_nanos();

    let ts = Timestamp::from_unix(&context, secs, nanos);

    (0..size).map(|_| UUID::new_v7_with_timestamp(ts)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_uuid_v4_creation() {
        let uuid = UUID::new_v4();
        assert_eq!(uuid.0.get_version_num(), 4, "UUID version should be 4");
    }

    #[test]
    fn test_uuid_v7_creation() {
        let uuid = UUID::new_v7();
        assert_eq!(uuid.0.get_version_num(), 7, "UUID version should be 7");
    }

    #[test]
    fn test_uuid_v7_with_timestamp() {
        let context = ContextV7::new();
        let ts = Timestamp::from_unix(&context, 1_700_000_000, 123456789);
        let uuid = UUID::new_v7_with_timestamp(ts);
        assert_eq!(uuid.0.get_version_num(), 7);
    }

    #[test]
    fn test_uuid_default() {
        let uuid: UUID = Default::default();
        assert_eq!(
            uuid.0.get_version_num(),
            7,
            "Default UUID should be version 7"
        );
    }

    #[test]
    fn test_uuid_formatting() {
        let uuid = UUID::new_v4();
        let simple = uuid.simple();
        let hyphenated = uuid.hyphenated();

        assert_eq!(simple.len(), 32, "Simple format should have 32 characters");
        assert_eq!(
            hyphenated.len(),
            36,
            "Hyphenated format should have 36 characters"
        );
    }

    #[test]
    fn test_uuid_from_str() {
        let uuid = UUID::new_v4();
        let s = uuid.hyphenated();
        let parsed = UUID::from_str(&s).expect("Parsing should succeed");
        assert_eq!(uuid, parsed, "Parsed UUID should match original");
    }

    #[test]
    fn test_batch_uuid_v4_unique() {
        let uuids = batch_uuid_v4(1000);
        let set: HashSet<_> = uuids.iter().collect();
        assert_eq!(
            set.len(),
            uuids.len(),
            "All UUIDs in batch_v4 should be unique"
        );
    }

    #[test]
    fn test_batch_uuid_v7_unique() {
        let uuids = batch_uuid_v7(1000);
        let set: HashSet<_> = uuids.iter().collect();
        assert_eq!(
            set.len(),
            uuids.len(),
            "All UUIDs in batch_v7 should be unique"
        );
    }

    #[test]
    fn test_parse_uuid_versions() {
        let uuids = [
            // v4
            ("df5bb533-99ea-4e39-b35e-919509bce87f", 4),
            ("DF5BB533-99EA-4E39-B35E-919509BCE87F", 4),
            ("df5bb53399ea4e39b35e919509bce87f", 4),
            ("DF5BB53399EA4E39B35E919509BCE87F", 4),
            // v7
            ("019a7cd5-4565-7e58-a414-5f49e76c5949", 7),
            ("019A7CD5-4565-7E58-A414-5F49E76C5949", 7),
            ("019a7cd545657e58a4145f49e76c5949", 7),
            ("019A7CD545657E58A4145F49E76C5949", 7),
        ];

        for &(s, expected_version) in &uuids {
            let parsed =
                UUID::from_str(s).unwrap_or_else(|_| panic!("Parsing '{}' should succeed", s));
            assert_eq!(
                parsed.0.get_version_num(),
                expected_version,
                "UUID '{}' should be version {}",
                s,
                expected_version
            );
        }
    }
}
