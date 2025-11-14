use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataFeedType {
    #[serde(rename = "ITCH")]
    Itch,
    #[serde(rename = "MDF")]
    Mdf,
}

impl DataFeedType {
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            DataFeedType::Itch => "ITCH",
            DataFeedType::Mdf => "MDF",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str() {
        assert_eq!(DataFeedType::Itch.as_str(), "ITCH");
        assert_eq!(DataFeedType::Mdf.as_str(), "MDF");
    }
}
