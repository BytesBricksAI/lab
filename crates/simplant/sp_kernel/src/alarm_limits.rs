//! Alarm limit sets for process variables.

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, Result};
use crate::range::EngineeringRange;
use crate::unit::UnitOfMeasure;

/// Four-level alarm limits (low-low, low, high, high-high) with a shared unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AlarmLimits {
    low_low: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
    high_high: Option<f64>,
    unit: UnitOfMeasure,
}

impl AlarmLimits {
    /// Creates alarm limits, validating ordering and finiteness.
    pub fn new(
        low_low: Option<f64>,
        low: Option<f64>,
        high: Option<f64>,
        high_high: Option<f64>,
        unit: UnitOfMeasure,
    ) -> Result<Self> {
        if low_low.is_none() && low.is_none() && high.is_none() && high_high.is_none() {
            return Err(KernelError::EmptyAlarmLimits);
        }

        let present = [low_low, low, high, high_high];
        for value in present.into_iter().flatten() {
            if !value.is_finite() {
                return Err(KernelError::InvalidAlarmLimits(
                    "all limits must be finite".to_owned(),
                ));
            }
        }

        if let (Some(ll), Some(l)) = (low_low, low)
            && ll > l
        {
            return Err(KernelError::InvalidAlarmLimits(
                "low_low must be less than or equal to low".to_owned(),
            ));
        }

        if let (Some(h), Some(hh)) = (high, high_high)
            && h > hh
        {
            return Err(KernelError::InvalidAlarmLimits(
                "high must be less than or equal to high_high".to_owned(),
            ));
        }

        let low_half_max = [low_low, low].into_iter().flatten().reduce(f64::max);
        let high_half_min = [high, high_high].into_iter().flatten().reduce(f64::min);

        if let (Some(low_max), Some(high_min)) = (low_half_max, high_half_min)
            && low_max >= high_min
        {
            return Err(KernelError::InvalidAlarmLimits(
                "lower alarm limits must be strictly below upper alarm limits".to_owned(),
            ));
        }

        let ordered_present: Vec<f64> = [low_low, low, high, high_high]
            .into_iter()
            .flatten()
            .collect();
        for window in ordered_present.windows(2) {
            if window[0] > window[1] {
                return Err(KernelError::InvalidAlarmLimits(
                    "limits must be non-decreasing in LL, L, H, HH order".to_owned(),
                ));
            }
        }

        Ok(Self {
            low_low,
            low,
            high,
            high_high,
            unit,
        })
    }

    /// Low-low alarm limit, if set.
    pub fn low_low(&self) -> Option<f64> {
        self.low_low
    }

    /// Low alarm limit, if set.
    pub fn low(&self) -> Option<f64> {
        self.low
    }

    /// High alarm limit, if set.
    pub fn high(&self) -> Option<f64> {
        self.high
    }

    /// High-high alarm limit, if set.
    pub fn high_high(&self) -> Option<f64> {
        self.high_high
    }

    /// Unit associated with all limits.
    pub fn unit(&self) -> UnitOfMeasure {
        self.unit
    }

    /// Returns `true` if every present limit lies within `range` (same unit required).
    pub fn within(&self, range: &EngineeringRange) -> bool {
        if self.unit != range.unit() {
            return false;
        }

        [self.low_low, self.low, self.high, self.high_high]
            .into_iter()
            .flatten()
            .all(|limit| range.contains(limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_valid_limits() {
        assert!(
            AlarmLimits::new(
                Some(10.0),
                Some(20.0),
                Some(80.0),
                Some(90.0),
                UnitOfMeasure::Bar,
            )
            .is_ok()
        );
    }

    #[test]
    fn invalid_low_low_greater_than_low() {
        let err = AlarmLimits::new(
            Some(20.0),
            Some(10.0),
            Some(80.0),
            Some(90.0),
            UnitOfMeasure::Bar,
        )
        .unwrap_err();
        assert!(matches!(err, KernelError::InvalidAlarmLimits(_)));
    }

    #[test]
    fn only_high_limits_ok() {
        assert!(AlarmLimits::new(None, None, Some(80.0), Some(90.0), UnitOfMeasure::Bar).is_ok());
    }

    #[test]
    fn all_none_is_error() {
        assert_eq!(
            AlarmLimits::new(None, None, None, None, UnitOfMeasure::Bar).unwrap_err(),
            KernelError::EmptyAlarmLimits,
        );
    }

    #[test]
    fn within_range() {
        let limits = AlarmLimits::new(
            Some(10.0),
            Some(20.0),
            Some(80.0),
            Some(90.0),
            UnitOfMeasure::Bar,
        )
        .unwrap();
        let range = EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap();
        assert!(limits.within(&range));

        let narrow = EngineeringRange::new(15.0, 85.0, UnitOfMeasure::Bar).unwrap();
        assert!(!limits.within(&narrow));

        let wrong_unit = EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Psi).unwrap();
        assert!(!limits.within(&wrong_unit));
    }
}
