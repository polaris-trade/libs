use std::time::Instant;

use opentelemetry::Context;

#[derive(Debug, Clone)]
pub struct TraceData {
    pub recv_at: Instant,
    pub ctx: Context,
}

impl Default for TraceData {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceData {
    #[inline]
    pub fn new() -> Self {
        Self {
            recv_at: std::time::Instant::now(),
            ctx: Context::new(),
        }
    }

    #[inline]
    pub fn with_current_context() -> Self {
        Self {
            recv_at: std::time::Instant::now(),
            ctx: Context::current(),
        }
    }

    #[inline]
    pub fn elapsed_micros(&self) -> u64 {
        self.recv_at.elapsed().as_micros() as u64
    }

    #[inline]
    pub fn elapsed_millis(&self) -> u64 {
        self.recv_at.elapsed().as_millis() as u64
    }

    #[inline]
    pub fn elapsed_nanos(&self) -> u64 {
        self.recv_at.elapsed().as_nanos() as u64
    }
}
