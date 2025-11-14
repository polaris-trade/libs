use crate::{result::ParseResult, utils::check_len};
use core::str::from_utf8_unchecked;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
};
use std::fmt;

struct AlphaVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for AlphaVisitor<N> {
    type Value = Alpha<N>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an ASCII string up to length {}", N)
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        let bytes = value.as_bytes();
        if bytes.len() > N {
            return Err(E::custom(format!(
                "expected at most length {}, got {}",
                N,
                bytes.len()
            )));
        }

        let mut arr = [b' '; N];
        arr[..bytes.len()].copy_from_slice(bytes);
        Ok(Alpha::new(arr))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct Alpha<const N: usize> {
    bytes: [u8; N],
    len: u8,
}

impl<const N: usize> Alpha<N> {
    #[inline(always)]
    pub fn new(bytes: [u8; N]) -> Self {
        let mut end = N;
        while end > 0 && bytes[end - 1] == b' ' {
            end -= 1;
        }

        Self {
            bytes,
            len: end as u8,
        }
    }

    #[inline(always)]
    pub fn parse(input: &[u8]) -> ParseResult<Self> {
        check_len(input, N)?;

        let mut buf = [0u8; N];
        buf.copy_from_slice(&input[..N]);
        Ok(Self::new(buf))
    }

    /// Returns the full underlying ASCII string (including padding).
    ///
    /// # Safety
    /// Safe because upstream ensures only ASCII bytes are sent.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe {
            debug_assert!(self.bytes.iter().all(|&b| b.is_ascii()));
            from_utf8_unchecked(&self.bytes)
        }
    }

    /// Returns the trimmed ASCII string (O(1) due to cached len).
    ///
    /// # Safety
    /// Safe because upstream ensures only ASCII bytes are sent.
    #[inline(always)]
    pub fn as_trimmed_str(&self) -> &str {
        unsafe {
            debug_assert!(self.bytes.iter().all(|&b| b.is_ascii()));
            from_utf8_unchecked(&self.bytes[..self.len as usize])
        }
    }

    /// Returns the full byte slice (including padding).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }

    /// Returns the trimmed length.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Checks if the Alpha value is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> AsRef<str> for Alpha<N> {
    fn as_ref(&self) -> &str {
        self.as_trimmed_str()
    }
}

impl<const N: usize> core::fmt::Debug for Alpha<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Alpha")
            .field(&self.as_trimmed_str())
            .finish()
    }
}

impl<const N: usize> core::fmt::Display for Alpha<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_trimmed_str())
    }
}

impl<const N: usize> Serialize for Alpha<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_trimmed_str())
    }
}

impl<'de, const N: usize> Deserialize<'de> for Alpha<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(AlphaVisitor::<N>)
    }
}

pub type Alpha3 = Alpha<3>;
pub type Alpha4 = Alpha<4>;
pub type Alpha8 = Alpha<8>;
pub type Alpha10 = Alpha<10>;
pub type Alpha12 = Alpha<12>;
pub type Alpha16 = Alpha<16>;
pub type Alpha32 = Alpha<32>;
pub type Alpha40 = Alpha<40>;
pub type Alpha64 = Alpha<64>;
pub type Alpha100 = Alpha<100>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParseError;
    use serde_json;

    #[test]
    fn test_alpha_new() {
        let bytes = *b"DATA";
        let alpha = Alpha4::new(bytes);
        assert_eq!(alpha.as_bytes(), b"DATA");
        assert_eq!(alpha.as_str(), "DATA");
        assert_eq!(alpha.as_trimmed_str(), "DATA");
    }

    #[test]
    fn test_alpha_parse_valid() {
        let bytes = b"ABCD";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.as_bytes(), b"ABCD");
        assert_eq!(alpha.as_str(), "ABCD");
        assert_eq!(alpha.as_trimmed_str(), "ABCD");
    }

    #[test]
    fn test_alpha_parse_short_buffer() {
        let bytes = b"ABC"; // only 3 bytes, need 4
        let result = Alpha4::parse(bytes).unwrap_err();
        assert!(matches!(result, ParseError::Incomplete { needed: Some(1) }));
    }

    #[test]
    fn test_alpha_trim_trailing_spaces() {
        let bytes = b"HI  ";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.as_str(), "HI  ");
        assert_eq!(alpha.as_trimmed_str(), "HI");
    }

    #[test]
    fn test_alpha_as_str_includes_spaces() {
        let bytes = b"XY Z";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.as_str(), "XY Z");
        assert_eq!(alpha.as_trimmed_str(), "XY Z"); // no trailing spaces
    }

    #[test]
    fn test_alpha_display_and_debug() {
        let bytes = b"TEST";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(format!("{}", alpha), "TEST");
        assert_eq!(format!("{:?}", alpha), "Alpha(\"TEST\")");
    }

    #[test]
    fn test_alpha_with_spaces_only() {
        let bytes = b"    ";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.as_trimmed_str(), "");
        assert_eq!(format!("{}", alpha), "");
    }

    #[test]
    fn test_alpha_larger_sizes() {
        let bytes = b"ABCDEFGH";
        let alpha8 = Alpha8::parse(bytes).unwrap();
        assert_eq!(alpha8.as_trimmed_str(), "ABCDEFGH");

        let bytes10 = b"HELLOWORLD";
        let alpha10 = Alpha10::parse(bytes10).unwrap();
        assert_eq!(alpha10.as_trimmed_str(), "HELLOWORLD");
    }

    #[test]
    fn test_alpha_empty() {
        let bytes = b"    ";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert!(alpha.is_empty());
        assert_eq!(alpha.len(), 0);
    }

    #[test]
    fn test_alpha_partial_spaces() {
        let bytes = b"AB  ";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.as_trimmed_str(), "AB");
        assert_eq!(alpha.len(), 2);
    }

    #[test]
    fn test_alpha_full_length() {
        let bytes = b"ABCD";
        let alpha = Alpha4::parse(bytes).unwrap();
        assert_eq!(alpha.len(), 4);
    }

    #[test]
    fn test_alpha_serialize_deserialize() {
        let alpha = Alpha4::parse(b"TEST").unwrap();
        let json = serde_json::to_string(&alpha).unwrap();
        assert_eq!(json, "\"TEST\"");

        let de: Alpha4 = serde_json::from_str(&json).unwrap();
        assert_eq!(de.as_trimmed_str(), "TEST");
    }

    #[test]
    fn test_alpha_deserialize_too_long() {
        let json = "\"TOOLONG\""; // 7 characters for Alpha4
        let result: Result<Alpha4, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_alpha_visitor_expecting_covered() {
        // This should trigger `visit_*` methods other than visit_str,
        // causing Serde to fallback and call `expecting`
        let json = "123"; // a number, not a string
        let result: Result<Alpha4, _> = serde_json::from_str(json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("an ASCII string up to length 4"));
    }

    #[test]
    fn test_alpha_as_ref() {
        let bytes = *b"REF ";
        let alpha = Alpha4::new(bytes);
        let s: &str = alpha.as_ref();
        assert_eq!(s, "REF");
    }
}
