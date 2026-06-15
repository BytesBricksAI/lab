//! Physical dimensions and units of measure.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Physical dimension of a process variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dimension {
    /// Pressure.
    Pressure,
    /// Temperature.
    Temperature,
    /// Volumetric flow rate.
    VolumetricFlow,
    /// Mass flow rate.
    MassFlow,
    /// Length.
    Length,
    /// Dimensionless quantity.
    Dimensionless,
}

/// Engineering unit of measure with SI-base conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitOfMeasure {
    /// Kilopascal (kPa).
    Kilopascal,
    /// Bar.
    Bar,
    /// Pounds per square inch (psi).
    Psi,
    /// Megapascal (`MPa`).
    Megapascal,
    /// Degrees Celsius (°C).
    DegreeCelsius,
    /// Kelvin (K).
    Kelvin,
    /// Cubic meters per hour (m³/h).
    CubicMeterPerHour,
    /// Barrels per day (bbl/d).
    BarrelPerDay,
    /// Kilograms per hour (kg/h).
    KilogramPerHour,
    /// Meter (m).
    Meter,
    /// Percent (%).
    Percent,
    /// Unitless ratio.
    Ratio,
}

impl UnitOfMeasure {
    /// Returns the physical dimension of this unit.
    pub fn dimension(&self) -> Dimension {
        match self {
            Self::Kilopascal | Self::Bar | Self::Psi | Self::Megapascal => Dimension::Pressure,
            Self::DegreeCelsius | Self::Kelvin => Dimension::Temperature,
            Self::CubicMeterPerHour | Self::BarrelPerDay => Dimension::VolumetricFlow,
            Self::KilogramPerHour => Dimension::MassFlow,
            Self::Meter => Dimension::Length,
            Self::Percent | Self::Ratio => Dimension::Dimensionless,
        }
    }

    /// Returns the conventional symbol for this unit.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Kilopascal => "kPa",
            Self::Bar => "bar",
            Self::Psi => "psi",
            Self::Megapascal => "MPa",
            Self::DegreeCelsius => "°C",
            Self::Kelvin => "K",
            Self::CubicMeterPerHour => "m³/h",
            Self::BarrelPerDay => "bbl/d",
            Self::KilogramPerHour => "kg/h",
            Self::Meter => "m",
            Self::Percent => "%",
            Self::Ratio => "",
        }
    }

    /// Converts `value` in this unit to the SI base unit of its dimension.
    #[expect(clippy::wrong_self_convention)]
    pub fn to_base(&self, value: f64) -> f64 {
        match self {
            Self::Kilopascal => value * 1_000.0,
            Self::Bar => value * 100_000.0,
            Self::Psi => value * 6_894.757_293_168,
            Self::Megapascal => value * 1_000_000.0,
            Self::DegreeCelsius => value + 273.15,
            Self::Kelvin | Self::Meter | Self::Ratio => value,
            Self::CubicMeterPerHour | Self::KilogramPerHour => value / 3_600.0,
            Self::BarrelPerDay => value * 0.158_987_294_928 / 86_400.0,
            Self::Percent => value / 100.0,
        }
    }

    /// Converts a SI base-unit value into this unit.
    #[expect(clippy::wrong_self_convention)]
    pub fn from_base(&self, base_value: f64) -> f64 {
        match self {
            Self::Kilopascal => base_value / 1_000.0,
            Self::Bar => base_value / 100_000.0,
            Self::Psi => base_value / 6_894.757_293_168,
            Self::Megapascal => base_value / 1_000_000.0,
            Self::DegreeCelsius => base_value - 273.15,
            Self::Kelvin | Self::Meter | Self::Ratio => base_value,
            Self::CubicMeterPerHour | Self::KilogramPerHour => base_value * 3_600.0,
            Self::BarrelPerDay => base_value * 86_400.0 / 0.158_987_294_928,
            Self::Percent => base_value * 100.0,
        }
    }

    /// Returns `true` if `other` shares the same physical dimension.
    pub fn same_dimension(&self, other: &Self) -> bool {
        self.dimension() == other.dimension()
    }
}

impl fmt::Display for UnitOfMeasure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        let denom = a.abs().max(b.abs()).max(1.0);
        (a - b).abs() / denom <= 1e-6
    }

    #[test]
    fn bar_to_base() {
        assert!(approx_eq(UnitOfMeasure::Bar.to_base(1.0), 100_000.0));
    }

    #[test]
    fn kilopascal_to_base() {
        assert!(approx_eq(UnitOfMeasure::Kilopascal.to_base(1.0), 1_000.0));
    }

    #[test]
    fn celsius_freezing_to_kelvin() {
        assert!(approx_eq(UnitOfMeasure::DegreeCelsius.to_base(0.0), 273.15));
    }

    #[test]
    fn round_trip_conversions() {
        let units = [
            UnitOfMeasure::Bar,
            UnitOfMeasure::Psi,
            UnitOfMeasure::Megapascal,
            UnitOfMeasure::DegreeCelsius,
            UnitOfMeasure::CubicMeterPerHour,
            UnitOfMeasure::BarrelPerDay,
            UnitOfMeasure::Percent,
        ];
        let values = [0.0, 1.0, 42.5, -17.3, 100.0];

        for unit in units {
            for value in values {
                let base = unit.to_base(value);
                let round_trip = unit.from_base(base);
                assert!(
                    approx_eq(value, round_trip),
                    "round-trip failed for {unit:?} at {value}"
                );
            }
        }
    }

    #[test]
    fn same_dimension_checks() {
        assert!(UnitOfMeasure::Bar.same_dimension(&UnitOfMeasure::Psi));
        assert!(!UnitOfMeasure::Bar.same_dimension(&UnitOfMeasure::Kelvin));
    }
}
