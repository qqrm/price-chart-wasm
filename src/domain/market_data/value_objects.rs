use derive_more::{Constructor, Deref, DerefMut, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum::{AsRefStr, Display as StrumDisplay, EnumIter, EnumString};

/// Value Object - Price with generated helpers
#[derive(
    Debug, Clone, Copy, PartialEq, From, Into, Deref, DerefMut, Constructor, Serialize, Deserialize,
)]
pub struct Price(f64);

impl Price {
    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn validate(value: f64) -> Result<Self, String> {
        if value.is_finite() && value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(format!("Invalid price: {}", value))
        }
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Value Object - Volume with generated helpers
#[derive(
    Debug, Clone, Copy, PartialEq, From, Into, Deref, DerefMut, Constructor, Serialize, Deserialize,
)]
pub struct Volume(f64);

impl Volume {
    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn validate(value: f64) -> Result<Self, String> {
        if value.is_finite() && value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(format!("Invalid volume: {}", value))
        }
    }
}

/// Value Object - Timestamp with generated helpers
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    From,
    Into,
    Deref,
    DerefMut,
    Constructor,
    Serialize,
    Deserialize,
)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Create from milliseconds (for compatibility)
    pub fn from_millis(value: u64) -> Self {
        Self(value)
    }

    pub fn from_milliseconds(millis: u64) -> Result<Self, String> {
        Ok(Self(millis))
    }
}

/// Value Object - OHLCV data with generated helpers
#[derive(Debug, Clone, Copy, PartialEq, Constructor, Serialize, Deserialize)]
pub struct OHLCV {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Volume,
}

impl OHLCV {
    /// Check the validity of OHLCV data
    pub fn is_valid(&self) -> bool {
        self.high >= self.open
            && self.high >= self.close
            && self.high >= self.low
            && self.low <= self.open
            && self.low <= self.close
            && self.volume.value() >= 0.0
    }
}

/// Value Object - Trading symbol with generated helpers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut, Display, Serialize, Deserialize)]
#[display(fmt = "Symbol({})", _0)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(symbol: String) -> Result<Self, String> {
        if symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }
        Ok(Self(symbol.to_uppercase()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Self(value.to_uppercase())
    }
}

/// List of supported trading symbols
pub fn default_symbols() -> Vec<Symbol> {
    vec![Symbol::from("BTCUSDT"), Symbol::from("ETHUSDT"), Symbol::from("SOLUSDT")]
}

/// Value Object - Time interval (only required variants)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    StrumDisplay,
    EnumIter,
    EnumString,
    AsRefStr,
    Serialize,
    Deserialize,
)]
pub enum TimeInterval {
    #[strum(serialize = "2s")]
    #[serde(rename = "2s")]
    TwoSeconds,

    #[strum(serialize = "1m")]
    #[serde(rename = "1m")]
    OneMinute,

    #[strum(serialize = "5m")]
    #[serde(rename = "5m")]
    FiveMinutes,

    #[strum(serialize = "15m")]
    #[serde(rename = "15m")]
    FifteenMinutes,

    #[strum(serialize = "1h")]
    #[serde(rename = "1h")]
    OneHour,

    #[strum(serialize = "4h")]
    #[serde(rename = "4h")]
    FourHours,

    #[strum(serialize = "1d")]
    #[serde(rename = "1d")]
    OneDay,

    #[strum(serialize = "1w")]
    #[serde(rename = "1w")]
    OneWeek,

    #[strum(serialize = "1M")]
    #[serde(rename = "1M")]
    OneMonth,
}

impl TimeInterval {
    pub fn to_binance_str(&self) -> &str {
        self.as_ref()
    }

    pub fn duration_ms(&self) -> u64 {
        match self {
            Self::TwoSeconds => 2 * 1000,
            Self::OneMinute => 60 * 1000,
            Self::FiveMinutes => 5 * 60 * 1000,
            Self::FifteenMinutes => 15 * 60 * 1000,
            Self::OneHour => 60 * 60 * 1000,
            Self::FourHours => 4 * 60 * 60 * 1000,
            Self::OneDay => 24 * 60 * 60 * 1000,
            Self::OneWeek => 7 * 24 * 60 * 60 * 1000,
            Self::OneMonth => 30 * 24 * 60 * 60 * 1000,
        }
    }
}
