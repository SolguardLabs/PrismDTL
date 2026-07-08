use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{PrismError, PrismResult};

pub const MAX_PRISM_SUPPLY: u128 = 10_000_000_000_000;

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Amount(u128);

impl Amount {
    pub const ZERO: Self = Self(0);

    pub fn new(units: u128) -> PrismResult<Self> {
        if units > MAX_PRISM_SUPPLY {
            return Err(PrismError::AmountOutOfRange {
                value: units,
                max: MAX_PRISM_SUPPLY,
            });
        }

        Ok(Self(units))
    }

    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub const fn units(self) -> u128 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Self) -> PrismResult<Self> {
        let value = self
            .0
            .checked_add(rhs.0)
            .ok_or(PrismError::AmountOverflow)?;
        Self::new(value)
    }

    pub fn checked_sub(self, rhs: Self) -> PrismResult<Self> {
        let value = self
            .0
            .checked_sub(rhs.0)
            .ok_or(PrismError::AmountUnderflow)?;
        Self::new(value)
    }
}

impl TryFrom<u128> for Amount {
    type Error = PrismError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
