use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const NO_PRICE: i64 = i64::MIN;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct Price {
    raw: i64,
    decimals: u8,
}

impl Price {
    #[inline]
    pub const fn new(raw: i64) -> Self {
        Self { raw, decimals: 0 }
    }

    #[inline]
    pub const fn new_with_decimals(raw: i64, decimals: u8) -> Self {
        Self { raw, decimals }
    }

    #[inline]
    pub fn raw(self) -> i64 {
        self.raw
    }

    #[inline]
    pub fn decimals(self) -> u8 {
        self.decimals
    }

    #[inline]
    pub fn is_none(self) -> bool {
        self.raw == NO_PRICE
    }

    /// Set decimals after creation (e.g. once Order Book Directory is known)
    #[inline]
    pub fn set_decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    /// Convert to Decimal if possible (both raw != NO_PRICE and decimals known)
    #[inline]
    pub fn as_decimal(self) -> Option<Decimal> {
        if self.is_none() {
            return None;
        }

        Some(Decimal::from_i128_with_scale(
            self.raw as i128,
            self.decimals as u32,
        ))
    }
}

impl From<i64> for Price {
    fn from(v: i64) -> Self {
        Self::new(v)
    }
}

impl From<Price> for i64 {
    fn from(p: Price) -> Self {
        p.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_new() {
        let p = Price::new(100);
        assert_eq!(p.raw(), 100);
        assert_eq!(p.decimals(), 0);
        assert!(!p.is_none());
    }

    #[test]
    fn test_new_with_decimals() {
        let p = Price::new_with_decimals(12345, 2);
        assert_eq!(p.raw(), 12345);
        assert_eq!(p.decimals(), 2);
        assert!(!p.is_none());
    }

    #[test]
    fn test_is_none() {
        let p = Price::new(NO_PRICE);
        assert!(p.is_none());

        let p2 = Price::new(0);
        assert!(!p2.is_none());
    }

    #[test]
    fn test_set_decimals() {
        let p = Price::new(100).set_decimals(3);
        assert_eq!(p.decimals(), 3);
        assert_eq!(p.raw(), 100);
    }

    #[test]
    fn test_as_decimal() {
        let p = Price::new_with_decimals(12345, 2);
        let dec = p.as_decimal().unwrap();
        assert_eq!(dec, Decimal::new(12345, 2));

        let none_price = Price::new(NO_PRICE);
        assert!(none_price.as_decimal().is_none());
    }

    #[test]
    fn test_from_i64() {
        let p: Price = 999i64.into();
        assert_eq!(p.raw(), 999);
        assert_eq!(p.decimals(), 0);
    }

    #[test]
    fn test_into_i64() {
        let p = Price::new_with_decimals(777, 2);
        let raw: i64 = p.into();
        assert_eq!(raw, 777);
    }
}
