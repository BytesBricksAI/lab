//! Steady-state unit-operation calculations without thermodynamics (F6, termo-free subset).
//!
//! Supported operations: mixer, splitter, valve, pump, and pipe. Heater, cooler, and flash drum
//! require rigorous energy and phase equilibrium models and are out of scope here.

/// Tolerance for composition fractions summing to one.
const COMPOSITION_TOLERANCE: f64 = 1e-6;

/// Errors produced by [`StreamState`] validation and unit-operation calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitOpError {
    /// Mass flow must be non-negative.
    NegativeMassFlow,

    /// Pressure must be strictly positive.
    NonPositivePressure,

    /// Temperature must be strictly positive.
    NonPositiveTemperature,

    /// Composition vector must not be empty.
    EmptyComposition,

    /// Composition fractions do not sum to one within tolerance.
    CompositionNotNormalized { sum: f64 },

    /// A composition fraction is negative.
    NegativeCompositionFraction,

    /// Mixer requires at least one inlet stream.
    EmptyInlets,

    /// Inlet streams have composition vectors of different lengths.
    InletCompositionArityMismatch { expected: usize, found: usize },

    /// Total mass flow through the mixer is zero.
    ZeroTotalMassFlow,

    /// Splitter requires at least one outlet fraction.
    EmptySplitFractions,

    /// A split fraction is negative.
    InvalidSplitFraction { index: usize, value: f64 },

    /// Split fractions do not sum to one within tolerance.
    SplitFractionsNotNormalized { sum: f64 },

    /// Valve outlet pressure exceeds inlet pressure.
    OutletPressureExceedsInlet { inlet: f64, outlet: f64 },

    /// Valve outlet pressure must be strictly positive.
    NonPositiveOutletPressure,

    /// Pump outlet pressure is below inlet pressure.
    OutletPressureBelowInlet { inlet: f64, outlet: f64 },

    /// Pump fluid density must be strictly positive.
    NonPositiveDensity,

    /// Pump efficiency must be in the interval (0, 1].
    InvalidEfficiency { value: f64 },

    /// Pipe pressure drop must be non-negative.
    NegativePressureDrop,

    /// Pipe outlet pressure would be non-positive.
    ResultingPressureNonPositive { inlet: f64, delta_p: f64 },
}

impl core::fmt::Display for UnitOpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NegativeMassFlow => write!(f, "mass flow must be non-negative"),
            Self::NonPositivePressure => write!(f, "pressure must be strictly positive"),
            Self::NonPositiveTemperature => write!(f, "temperature must be strictly positive"),
            Self::EmptyComposition => write!(f, "composition must not be empty"),
            Self::CompositionNotNormalized { sum } => {
                write!(f, "composition is not normalized (sum = {sum})")
            }
            Self::NegativeCompositionFraction => {
                write!(f, "composition fractions must be non-negative")
            }
            Self::EmptyInlets => write!(f, "mixer requires at least one inlet stream"),
            Self::InletCompositionArityMismatch { expected, found } => write!(
                f,
                "inlet composition arity mismatch (expected {expected}, found {found})"
            ),
            Self::ZeroTotalMassFlow => write!(f, "total inlet mass flow is zero"),
            Self::EmptySplitFractions => write!(f, "split fractions must not be empty"),
            Self::InvalidSplitFraction { index, value } => {
                write!(f, "split fraction at index {index} is negative ({value})")
            }
            Self::SplitFractionsNotNormalized { sum } => {
                write!(f, "split fractions do not sum to one (sum = {sum})")
            }
            Self::OutletPressureExceedsInlet { inlet, outlet } => write!(
                f,
                "valve outlet pressure ({outlet} Pa) exceeds inlet pressure ({inlet} Pa)"
            ),
            Self::NonPositiveOutletPressure => {
                write!(f, "outlet pressure must be strictly positive")
            }
            Self::OutletPressureBelowInlet { inlet, outlet } => write!(
                f,
                "pump outlet pressure ({outlet} Pa) is below inlet pressure ({inlet} Pa)"
            ),
            Self::NonPositiveDensity => write!(f, "fluid density must be strictly positive"),
            Self::InvalidEfficiency { value } => {
                write!(f, "pump efficiency must be in (0, 1], got {value}")
            }
            Self::NegativePressureDrop => write!(f, "pipe pressure drop must be non-negative"),
            Self::ResultingPressureNonPositive { inlet, delta_p } => write!(
                f,
                "pipe outlet pressure would be non-positive (inlet = {inlet} Pa, ΔP = {delta_p} Pa)"
            ),
        }
    }
}

impl std::error::Error for UnitOpError {}

/// Result type alias for unit-operation calculations.
pub type Result<T> = core::result::Result<T, UnitOpError>;

fn composition_sum(composition: &[f64]) -> f64 {
    composition.iter().sum()
}

fn validate_composition(composition: &[f64]) -> Result<()> {
    if composition.is_empty() {
        return Err(UnitOpError::EmptyComposition);
    }
    if composition.iter().any(|&x| x < 0.0) {
        return Err(UnitOpError::NegativeCompositionFraction);
    }
    let sum = composition_sum(composition);
    if (sum - 1.0).abs() > COMPOSITION_TOLERANCE {
        return Err(UnitOpError::CompositionNotNormalized { sum });
    }
    Ok(())
}

/// Thermodynamic state of a material stream (SI units).
///
/// All fields are validated on construction via [`StreamState::new`].
#[derive(Debug, Clone, PartialEq)]
pub struct StreamState {
    /// Mass flow rate in kg/s (≥ 0).
    mass_flow: f64,

    /// Absolute pressure in Pa (> 0).
    pressure: f64,

    /// Absolute temperature in K (> 0).
    temperature: f64,

    /// Mole or mass fractions (≥ 0 each), normalized to sum 1 ± [`COMPOSITION_TOLERANCE`].
    composition: Vec<f64>,
}

impl StreamState {
    /// Creates a stream state with validated fields.
    ///
    /// # Errors
    ///
    /// Returns [`UnitOpError`] if mass flow is negative, pressure or temperature is non-positive,
    /// composition is empty, contains negative fractions, or is not normalized.
    pub fn new(
        mass_flow: f64,
        pressure: f64,
        temperature: f64,
        composition: Vec<f64>,
    ) -> Result<Self> {
        if mass_flow < 0.0 {
            return Err(UnitOpError::NegativeMassFlow);
        }
        if pressure <= 0.0 {
            return Err(UnitOpError::NonPositivePressure);
        }
        if temperature <= 0.0 {
            return Err(UnitOpError::NonPositiveTemperature);
        }
        validate_composition(&composition)?;
        Ok(Self {
            mass_flow,
            pressure,
            temperature,
            composition,
        })
    }

    /// Mass flow rate in kg/s.
    pub fn mass_flow(&self) -> f64 {
        self.mass_flow
    }

    /// Absolute pressure in Pa.
    pub fn pressure(&self) -> f64 {
        self.pressure
    }

    /// Absolute temperature in K.
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Component fractions (normalized).
    pub fn composition(&self) -> &[f64] {
        &self.composition
    }
}

/// Mixes multiple inlet streams into a single outlet ([`UnitOpKind::Mixer`](sp_simulation::UnitOpKind::Mixer)).
///
/// Mass flow is the sum of inlet flows. Composition is a mass-flow-weighted average of inlet
/// compositions. Outlet pressure is the minimum inlet pressure. Temperature is a mass-flow-weighted
/// average — a valid approximation when no rigorous energy balance is applied.
///
/// # Errors
///
/// Returns an error if `inlets` is empty, inlet compositions differ in length, or total mass flow
/// is zero.
pub fn mix(inlets: &[StreamState]) -> Result<StreamState> {
    if inlets.is_empty() {
        return Err(UnitOpError::EmptyInlets);
    }

    let n_components = inlets[0].composition().len();
    let total_flow: f64 = inlets.iter().map(StreamState::mass_flow).sum();
    if total_flow == 0.0 {
        return Err(UnitOpError::ZeroTotalMassFlow);
    }

    for inlet in &inlets[1..] {
        let len = inlet.composition().len();
        if len != n_components {
            return Err(UnitOpError::InletCompositionArityMismatch {
                expected: n_components,
                found: len,
            });
        }
    }

    let mut blended = vec![0.0; n_components];
    let mut min_pressure = inlets[0].pressure();
    let mut weighted_temperature = 0.0;

    for inlet in inlets {
        let flow = inlet.mass_flow();
        for (i, &frac) in inlet.composition().iter().enumerate() {
            blended[i] += flow * frac;
        }
        min_pressure = min_pressure.min(inlet.pressure());
        weighted_temperature += flow * inlet.temperature();
    }

    for frac in &mut blended {
        *frac /= total_flow;
    }
    weighted_temperature /= total_flow;

    StreamState::new(total_flow, min_pressure, weighted_temperature, blended)
}

/// Splits one inlet stream into multiple outlets ([`UnitOpKind::Splitter`](sp_simulation::UnitOpKind::Splitter)).
///
/// Each outlet receives `inlet.mass_flow * fractions[j]` while pressure, temperature, and
/// composition are preserved.
///
/// # Errors
///
/// Returns an error if `fractions` is empty, any fraction is negative, or fractions do not sum
/// to one within tolerance.
pub fn split(inlet: &StreamState, fractions: &[f64]) -> Result<Vec<StreamState>> {
    if fractions.is_empty() {
        return Err(UnitOpError::EmptySplitFractions);
    }
    for (index, &frac) in fractions.iter().enumerate() {
        if frac < 0.0 {
            return Err(UnitOpError::InvalidSplitFraction { index, value: frac });
        }
    }
    let sum: f64 = fractions.iter().sum();
    if (sum - 1.0).abs() > COMPOSITION_TOLERANCE {
        return Err(UnitOpError::SplitFractionsNotNormalized { sum });
    }

    fractions
        .iter()
        .map(|&frac| {
            StreamState::new(
                inlet.mass_flow() * frac,
                inlet.pressure(),
                inlet.temperature(),
                inlet.composition().to_vec(),
            )
        })
        .collect()
}

/// Reduces stream pressure through a valve ([`UnitOpKind::Valve`](sp_simulation::UnitOpKind::Valve)).
///
/// Mass flow and composition are preserved. Temperature passes through unchanged — an isenthalpic
/// approximation valid without a rigorous thermodynamic model.
///
/// # Errors
///
/// Returns an error if `outlet_pressure` exceeds inlet pressure or is non-positive.
pub fn valve(inlet: &StreamState, outlet_pressure: f64) -> Result<StreamState> {
    if outlet_pressure <= 0.0 {
        return Err(UnitOpError::NonPositiveOutletPressure);
    }
    if outlet_pressure > inlet.pressure() {
        return Err(UnitOpError::OutletPressureExceedsInlet {
            inlet: inlet.pressure(),
            outlet: outlet_pressure,
        });
    }

    StreamState::new(
        inlet.mass_flow(),
        outlet_pressure,
        inlet.temperature(),
        inlet.composition().to_vec(),
    )
}

/// Raises stream pressure with a pump ([`UnitOpKind::Pump`](sp_simulation::UnitOpKind::Pump)).
///
/// Hydraulic work (watts):
///
/// ```text
/// W = ṁ · (P_out − P_in) / (ρ · η)
/// ```
///
/// Temperature and composition are preserved.
///
/// # Errors
///
/// Returns an error if `outlet_pressure` is below inlet pressure, `density` is non-positive, or
/// `efficiency` is not in (0, 1].
pub fn pump(
    inlet: &StreamState,
    outlet_pressure: f64,
    density: f64,
    efficiency: f64,
) -> Result<(StreamState, f64)> {
    if outlet_pressure < inlet.pressure() {
        return Err(UnitOpError::OutletPressureBelowInlet {
            inlet: inlet.pressure(),
            outlet: outlet_pressure,
        });
    }
    if density <= 0.0 {
        return Err(UnitOpError::NonPositiveDensity);
    }
    if efficiency <= 0.0 || efficiency > 1.0 {
        return Err(UnitOpError::InvalidEfficiency { value: efficiency });
    }

    let work_watts =
        inlet.mass_flow() * (outlet_pressure - inlet.pressure()) / (density * efficiency);
    let outlet = StreamState::new(
        inlet.mass_flow(),
        outlet_pressure,
        inlet.temperature(),
        inlet.composition().to_vec(),
    )?;
    Ok((outlet, work_watts))
}

/// Connects unit operations with a pressure drop ([`UnitOpKind::Pipe`](sp_simulation::UnitOpKind::Pipe)).
///
/// Outlet pressure is `P_in − ΔP`; mass flow, temperature, and composition are preserved.
///
/// # Errors
///
/// Returns an error if `delta_p` is negative or the resulting outlet pressure is non-positive.
pub fn pipe(inlet: &StreamState, delta_p: f64) -> Result<StreamState> {
    if delta_p < 0.0 {
        return Err(UnitOpError::NegativePressureDrop);
    }
    let outlet_pressure = inlet.pressure() - delta_p;
    if outlet_pressure <= 0.0 {
        return Err(UnitOpError::ResultingPressureNonPositive {
            inlet: inlet.pressure(),
            delta_p,
        });
    }

    StreamState::new(
        inlet.mass_flow(),
        outlet_pressure,
        inlet.temperature(),
        inlet.composition().to_vec(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    fn approx_slice_eq(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| approx_eq(*x, *y))
    }

    #[test]
    fn stream_state_rejects_negative_mass_flow() {
        let err = StreamState::new(-1.0, 1.0e5, 300.0, vec![1.0]).unwrap_err();
        assert_eq!(err, UnitOpError::NegativeMassFlow);
    }

    #[test]
    fn stream_state_rejects_non_positive_pressure() {
        let err = StreamState::new(1.0, 0.0, 300.0, vec![1.0]).unwrap_err();
        assert_eq!(err, UnitOpError::NonPositivePressure);
    }

    #[test]
    fn stream_state_rejects_non_positive_temperature() {
        let err = StreamState::new(1.0, 1.0e5, 0.0, vec![1.0]).unwrap_err();
        assert_eq!(err, UnitOpError::NonPositiveTemperature);
    }

    #[test]
    fn stream_state_rejects_empty_composition() {
        let err = StreamState::new(1.0, 1.0e5, 300.0, vec![]).unwrap_err();
        assert_eq!(err, UnitOpError::EmptyComposition);
    }

    #[test]
    fn stream_state_rejects_unnormalized_composition() {
        let err = StreamState::new(1.0, 1.0e5, 300.0, vec![0.3, 0.3]).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::CompositionNotNormalized { sum } if approx_eq(sum, 0.6)
        ));
    }

    #[test]
    fn stream_state_accepts_valid_state() {
        let stream = StreamState::new(2.0, 1.5e5, 320.0, vec![0.7, 0.3]).unwrap();
        assert!(approx_eq(stream.mass_flow(), 2.0));
        assert!(approx_eq(stream.pressure(), 1.5e5));
        assert!(approx_eq(stream.temperature(), 320.0));
        assert!(approx_slice_eq(stream.composition(), &[0.7, 0.3]));
    }

    #[test]
    fn mix_two_inlets_analytical() {
        // Inlet 1: 2 kg/s, P = 200 kPa, T = 300 K, z = [0.8, 0.2]
        // Inlet 2: 3 kg/s, P = 150 kPa, T = 350 K, z = [0.4, 0.6]
        // Expected: 5 kg/s, P = 150 kPa, T = 330 K, z = [0.56, 0.44]
        let inlet_a = StreamState::new(2.0, 200_000.0, 300.0, vec![0.8, 0.2]).unwrap();
        let inlet_b = StreamState::new(3.0, 150_000.0, 350.0, vec![0.4, 0.6]).unwrap();

        let outlet = mix(&[inlet_a, inlet_b]).unwrap();

        assert!(approx_eq(outlet.mass_flow(), 5.0));
        assert!(approx_eq(outlet.pressure(), 150_000.0));
        assert!(approx_eq(outlet.temperature(), 330.0));
        assert!(approx_slice_eq(outlet.composition(), &[0.56, 0.44]));
    }

    #[test]
    fn mix_rejects_empty_inlets() {
        let err = mix(&[]).unwrap_err();
        assert_eq!(err, UnitOpError::EmptyInlets);
    }

    #[test]
    fn mix_rejects_zero_total_flow() {
        let inlet = StreamState::new(0.0, 100_000.0, 300.0, vec![1.0]).unwrap();
        let err = mix(&[inlet]).unwrap_err();
        assert_eq!(err, UnitOpError::ZeroTotalMassFlow);
    }

    #[test]
    fn mix_rejects_composition_arity_mismatch() {
        let inlet_a = StreamState::new(1.0, 100_000.0, 300.0, vec![1.0]).unwrap();
        let inlet_b = StreamState::new(1.0, 100_000.0, 300.0, vec![0.5, 0.5]).unwrap();
        let err = mix(&[inlet_a, inlet_b]).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::InletCompositionArityMismatch {
                expected: 1,
                found: 2
            }
        ));
    }

    #[test]
    fn split_70_30_analytical() {
        // 10 kg/s inlet → 7 and 3 kg/s outlets, P/T/z preserved
        let inlet = StreamState::new(10.0, 250_000.0, 310.0, vec![0.6, 0.4]).unwrap();
        let outlets = split(&inlet, &[0.7, 0.3]).unwrap();

        assert_eq!(outlets.len(), 2);
        assert!(approx_eq(outlets[0].mass_flow(), 7.0));
        assert!(approx_eq(outlets[1].mass_flow(), 3.0));
        for outlet in &outlets {
            assert!(approx_eq(outlet.pressure(), 250_000.0));
            assert!(approx_eq(outlet.temperature(), 310.0));
            assert!(approx_slice_eq(outlet.composition(), &[0.6, 0.4]));
        }
    }

    #[test]
    fn split_rejects_invalid_fractions() {
        let inlet = StreamState::new(1.0, 100_000.0, 300.0, vec![1.0]).unwrap();
        let err = split(&inlet, &[0.6, 0.3]).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::SplitFractionsNotNormalized { sum } if approx_eq(sum, 0.9)
        ));
    }

    #[test]
    fn valve_reduces_pressure() {
        let inlet = StreamState::new(5.0, 300_000.0, 290.0, vec![1.0]).unwrap();
        let outlet = valve(&inlet, 200_000.0).unwrap();

        assert!(approx_eq(outlet.mass_flow(), 5.0));
        assert!(approx_eq(outlet.pressure(), 200_000.0));
        assert!(approx_eq(outlet.temperature(), 290.0));
        assert!(approx_slice_eq(outlet.composition(), &[1.0]));
    }

    #[test]
    fn valve_rejects_pressure_increase() {
        let inlet = StreamState::new(5.0, 200_000.0, 290.0, vec![1.0]).unwrap();
        let err = valve(&inlet, 250_000.0).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::OutletPressureExceedsInlet {
                inlet: 200_000.0,
                outlet: 250_000.0
            }
        ));
    }

    #[test]
    fn pump_work_analytical() {
        // ṁ = 2 kg/s, P_in = 100 kPa, P_out = 200 kPa, ρ = 1000 kg/m³, η = 0.8
        // W = 2 · (200000 − 100000) / (1000 · 0.8) = 250 W
        let inlet = StreamState::new(2.0, 100_000.0, 300.0, vec![1.0]).unwrap();
        let (outlet, work) = pump(&inlet, 200_000.0, 1000.0, 0.8).unwrap();

        assert!(approx_eq(work, 250.0));
        assert!(approx_eq(outlet.pressure(), 200_000.0));
        assert!(approx_eq(outlet.mass_flow(), 2.0));
        assert!(approx_eq(outlet.temperature(), 300.0));
    }

    #[test]
    fn pump_rejects_outlet_below_inlet() {
        let inlet = StreamState::new(2.0, 200_000.0, 300.0, vec![1.0]).unwrap();
        let err = pump(&inlet, 150_000.0, 1000.0, 0.8).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::OutletPressureBelowInlet {
                inlet: 200_000.0,
                outlet: 150_000.0
            }
        ));
    }

    #[test]
    fn pump_rejects_invalid_efficiency() {
        let inlet = StreamState::new(2.0, 100_000.0, 300.0, vec![1.0]).unwrap();
        let err = pump(&inlet, 200_000.0, 1000.0, 0.0).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::InvalidEfficiency { value } if approx_eq(value, 0.0)
        ));
    }

    #[test]
    fn pipe_pressure_drop_analytical() {
        let inlet = StreamState::new(4.0, 200_000.0, 305.0, vec![0.5, 0.5]).unwrap();
        let outlet = pipe(&inlet, 50_000.0).unwrap();

        assert!(approx_eq(outlet.pressure(), 150_000.0));
        assert!(approx_eq(outlet.mass_flow(), 4.0));
        assert!(approx_eq(outlet.temperature(), 305.0));
        assert!(approx_slice_eq(outlet.composition(), &[0.5, 0.5]));
    }

    #[test]
    fn pipe_rejects_non_positive_outlet_pressure() {
        let inlet = StreamState::new(4.0, 200_000.0, 305.0, vec![1.0]).unwrap();
        let err = pipe(&inlet, 200_000.0).unwrap_err();
        assert!(matches!(
            err,
            UnitOpError::ResultingPressureNonPositive {
                inlet: 200_000.0,
                delta_p: 200_000.0
            }
        ));
    }
}
