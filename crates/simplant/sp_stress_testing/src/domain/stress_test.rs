//! Stress test aggregate.

use serde::{Deserialize, Serialize};

use crate::domain::criteria::{AcceptanceCriterion, MeasuredOutcome};
use crate::domain::error::{Result, StressError};
use crate::domain::events::StressTestCompleted;
use crate::domain::events::StressTestPlanned;
use crate::domain::load_profile::{DesignLimit, LoadProfile, SafetyFactor};

/// Lifecycle state of a stress test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StressTestState {
    /// Planned and ready for execution.
    Planned,

    /// Evaluation completed (terminal).
    Completed,
}

impl StressTestState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::Completed => "Completed",
        }
    }
}

/// Aggregate root for a stress test over equipment and piping.
///
/// The load profile must not request loads above `design_limit × safety_factor` for
/// each variable. This invariant is enforced at planning time so invalid tests cannot
/// be constructed.
#[derive(Debug, Clone, PartialEq)]
pub struct StressTest {
    id: String,
    load_profile: LoadProfile,
    safety_factor: SafetyFactor,
    design_limits: Vec<DesignLimit>,
    acceptance_criteria: Vec<AcceptanceCriterion>,
    state: StressTestState,
}

impl StressTest {
    /// Plans a stress test after validating load profile bounds and acceptance criteria.
    pub fn plan(
        id: impl Into<String>,
        load_profile: LoadProfile,
        safety_factor: SafetyFactor,
        design_limits: Vec<DesignLimit>,
        acceptance_criteria: Vec<AcceptanceCriterion>,
    ) -> Result<(Self, StressTestPlanned)> {
        let id = id.into();
        if id.is_empty() {
            return Err(StressError::EmptyId);
        }
        if load_profile.is_empty() {
            return Err(StressError::EmptyLoadProfile);
        }
        if acceptance_criteria.is_empty() {
            return Err(StressError::EmptyAcceptanceCriteria);
        }

        let factor = safety_factor.value();
        for point in load_profile.points() {
            let limit = design_limits
                .iter()
                .find(|limit| limit.variable() == point.variable())
                .ok_or_else(|| StressError::UnmatchedLoadVariable(point.variable().to_owned()))?;

            let allowable = limit.max_value() * factor;
            if point.value() > allowable {
                return Err(StressError::LoadExceedsAllowable {
                    variable: point.variable().to_owned(),
                    load: point.value(),
                    allowable,
                });
            }
        }

        let point_count = load_profile.points().len();
        let test = Self {
            id: id.clone(),
            load_profile,
            safety_factor,
            design_limits,
            acceptance_criteria,
            state: StressTestState::Planned,
        };

        Ok((
            test,
            StressTestPlanned {
                test: id,
                point_count,
            },
        ))
    }

    /// Evaluates measured outcomes against acceptance criteria and transitions to `Completed`.
    ///
    /// Each criterion passes when a matching outcome exists and `outcome.value ≤ criterion.max_value`.
    /// Missing outcomes count as failures.
    pub fn evaluate(&mut self, outcomes: &[MeasuredOutcome]) -> Result<StressTestCompleted> {
        if self.state != StressTestState::Planned {
            return Err(StressError::InvalidStateTransition {
                from: self.state.as_str().to_owned(),
                to: StressTestState::Completed.as_str().to_owned(),
            });
        }

        let passed = self.acceptance_criteria.iter().all(|criterion| {
            outcomes
                .iter()
                .find(|outcome| outcome.metric() == criterion.metric())
                .is_some_and(|outcome| outcome.value() <= criterion.max_value())
        });

        self.state = StressTestState::Completed;

        Ok(StressTestCompleted {
            test: self.id.clone(),
            passed,
        })
    }

    /// Stress test identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current lifecycle state.
    pub fn state(&self) -> StressTestState {
        self.state
    }

    /// Planned load profile.
    pub fn load_profile(&self) -> &LoadProfile {
        &self.load_profile
    }

    /// Safety factor applied to design limits.
    pub fn safety_factor(&self) -> SafetyFactor {
        self.safety_factor
    }

    /// Design limits used for load validation.
    pub fn design_limits(&self) -> &[DesignLimit] {
        &self.design_limits
    }

    /// Acceptance criteria for evaluation.
    pub fn acceptance_criteria(&self) -> &[AcceptanceCriterion] {
        &self.acceptance_criteria
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::load_profile::LoadPoint;

    fn sample_limits() -> Vec<DesignLimit> {
        vec![
            DesignLimit::new("pressure", 100.0),
            DesignLimit::new("temperature", 200.0),
        ]
    }

    fn sample_profile() -> LoadProfile {
        LoadProfile::new(vec![
            LoadPoint::new("pressure", 120.0),
            LoadPoint::new("temperature", 250.0),
        ])
    }

    fn sample_criteria() -> Vec<AcceptanceCriterion> {
        vec![
            AcceptanceCriterion::new("max_pressure", 150.0),
            AcceptanceCriterion::new("max_temperature", 300.0),
        ]
    }

    #[test]
    fn plan_rejects_empty_id() {
        let result = StressTest::plan(
            "",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        );
        assert_eq!(result, Err(StressError::EmptyId));
    }

    #[test]
    fn plan_rejects_empty_load_profile() {
        let result = StressTest::plan(
            "ST-01",
            LoadProfile::new(vec![]),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        );
        assert_eq!(result, Err(StressError::EmptyLoadProfile));
    }

    #[test]
    fn plan_rejects_empty_acceptance_criteria() {
        let result = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            vec![],
        );
        assert_eq!(result, Err(StressError::EmptyAcceptanceCriteria));
    }

    #[test]
    fn plan_rejects_unmatched_load_variable() {
        let profile = LoadProfile::new(vec![LoadPoint::new("flow", 10.0)]);
        let result = StressTest::plan(
            "ST-01",
            profile,
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        );
        assert_eq!(
            result,
            Err(StressError::UnmatchedLoadVariable("flow".to_owned()))
        );
    }

    #[test]
    fn plan_rejects_load_exceeding_allowable() {
        let profile = LoadProfile::new(vec![LoadPoint::new("pressure", 200.0)]);
        let limits = vec![DesignLimit::new("pressure", 100.0)];
        let criteria = vec![AcceptanceCriterion::new("max_pressure", 150.0)];
        let result = StressTest::plan(
            "ST-01",
            profile,
            SafetyFactor::new(1.5).unwrap(),
            limits,
            criteria,
        );
        assert_eq!(
            result,
            Err(StressError::LoadExceedsAllowable {
                variable: "pressure".to_owned(),
                load: 200.0,
                allowable: 150.0,
            })
        );
    }

    #[test]
    fn plan_succeeds_within_allowable() {
        let (test, event) = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        )
        .unwrap();

        assert_eq!(test.state(), StressTestState::Planned);
        assert_eq!(test.id(), "ST-01");
        assert_eq!(
            event,
            StressTestPlanned {
                test: "ST-01".to_owned(),
                point_count: 2,
            }
        );
    }

    #[test]
    fn evaluate_passes_when_all_criteria_met() {
        let (mut test, _) = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        )
        .unwrap();

        let event = test
            .evaluate(&[
                MeasuredOutcome::new("max_pressure", 140.0),
                MeasuredOutcome::new("max_temperature", 290.0),
            ])
            .unwrap();

        assert!(event.passed);
        assert_eq!(test.state(), StressTestState::Completed);
    }

    #[test]
    fn evaluate_fails_when_outcome_exceeds_criterion() {
        let (mut test, _) = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        )
        .unwrap();

        let event = test
            .evaluate(&[
                MeasuredOutcome::new("max_pressure", 160.0),
                MeasuredOutcome::new("max_temperature", 290.0),
            ])
            .unwrap();

        assert!(!event.passed);
        assert_eq!(test.state(), StressTestState::Completed);
    }

    #[test]
    fn evaluate_fails_when_metric_missing() {
        let (mut test, _) = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        )
        .unwrap();

        let event = test
            .evaluate(&[MeasuredOutcome::new("max_pressure", 140.0)])
            .unwrap();

        assert!(!event.passed);
    }

    #[test]
    fn evaluate_rejects_invalid_state_transition() {
        let (mut test, _) = StressTest::plan(
            "ST-01",
            sample_profile(),
            SafetyFactor::new(1.5).unwrap(),
            sample_limits(),
            sample_criteria(),
        )
        .unwrap();

        let _ = test
            .evaluate(&[
                MeasuredOutcome::new("max_pressure", 140.0),
                MeasuredOutcome::new("max_temperature", 290.0),
            ])
            .unwrap();

        let result = test.evaluate(&[]);
        assert_eq!(
            result,
            Err(StressError::InvalidStateTransition {
                from: "Completed".to_owned(),
                to: "Completed".to_owned(),
            })
        );
    }
}
