//! Acquisition session aggregate.

use serde::{Deserialize, Serialize};
use sp_asset_model::AssetCatalog;

use crate::domain::binding::TagBinding;
use crate::domain::error::{AcquisitionError, Result};
use crate::domain::events::{AcquisitionStarted, AcquisitionStopped};
use crate::domain::sampling::SamplingPolicy;

/// Lifecycle state of an acquisition session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Created but not yet running.
    Created,

    /// Actively acquiring data.
    Running,

    /// Stopped (terminal).
    Stopped,
}

impl SessionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Running => "Running",
            Self::Stopped => "Stopped",
        }
    }
}

/// Aggregate root for a plant data acquisition session.
#[derive(Debug, Clone, PartialEq)]
pub struct AcquisitionSession {
    id: String,
    bindings: Vec<TagBinding>,
    policy: SamplingPolicy,
    state: SessionState,
}

impl AcquisitionSession {
    /// Creates a session after validating bindings against the asset catalog.
    pub fn create(
        id: impl Into<String>,
        bindings: Vec<TagBinding>,
        policy: SamplingPolicy,
        catalog: &AssetCatalog,
    ) -> Result<Self> {
        if bindings.is_empty() {
            return Err(AcquisitionError::EmptyBindings);
        }

        let mut seen = std::collections::HashSet::new();
        for binding in &bindings {
            if catalog.tag(binding.tag()).is_none() {
                return Err(AcquisitionError::UnknownTag(
                    binding.tag().as_str().to_owned(),
                ));
            }
            if !seen.insert(binding.tag().clone()) {
                return Err(AcquisitionError::DuplicateBinding(
                    binding.tag().as_str().to_owned(),
                ));
            }
        }

        Ok(Self {
            id: id.into(),
            bindings,
            policy,
            state: SessionState::Created,
        })
    }

    /// Transitions from `Created` to `Running` and returns the started event.
    pub fn start(&mut self) -> Result<AcquisitionStarted> {
        if self.state != SessionState::Created {
            return Err(AcquisitionError::InvalidStateTransition {
                from: self.state.as_str().to_owned(),
                to: SessionState::Running.as_str().to_owned(),
            });
        }
        self.state = SessionState::Running;
        Ok(AcquisitionStarted {
            session: self.id.clone(),
            tag_count: self.bindings.len(),
        })
    }

    /// Transitions from `Running` to `Stopped` and returns the stopped event.
    pub fn stop(&mut self, batches_recorded: u64) -> Result<AcquisitionStopped> {
        if self.state != SessionState::Running {
            return Err(AcquisitionError::InvalidStateTransition {
                from: self.state.as_str().to_owned(),
                to: SessionState::Stopped.as_str().to_owned(),
            });
        }
        self.state = SessionState::Stopped;
        Ok(AcquisitionStopped {
            session: self.id.clone(),
            batches_recorded,
        })
    }

    /// Session identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Tag bindings for this session.
    pub fn bindings(&self) -> &[TagBinding] {
        &self.bindings
    }

    /// Sampling policy for this session.
    pub fn policy(&self) -> SamplingPolicy {
        self.policy
    }

    /// Current lifecycle state.
    pub fn state(&self) -> SessionState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::binding::TagBinding;
    use sp_asset_model::{
        AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
        FacilityId, Tag, TagSpec, UnitId,
    };
    use sp_kernel::{EngineeringRange, TagId, UnitOfMeasure};

    fn sample_catalog() -> AssetCatalog {
        let (mut facility, _) = Facility::define(FacilityId::new("FAC-01").unwrap(), "Refinery");
        facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Crude")
            .unwrap();
        facility
            .add_unit(
                &AreaId::new("AREA-A").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "CDU",
            )
            .unwrap();

        let (equipment, _) = Equipment::commission(
            EquipmentId::new("EQ-101").unwrap(),
            UnitId::new("UNIT-100").unwrap(),
            "Separator",
            EquipmentKind::Vessel,
            DesignSpec::new(None, None).unwrap(),
        )
        .unwrap();

        let spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(spec).unwrap();

        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    fn binding(tag: &str, address: &str) -> TagBinding {
        TagBinding::new(TagId::new(tag).unwrap(), address).unwrap()
    }

    #[test]
    fn create_rejects_empty_bindings() {
        let catalog = sample_catalog();
        let err = AcquisitionSession::create("sess-1", vec![], SamplingPolicy::default(), &catalog)
            .unwrap_err();
        assert_eq!(err, AcquisitionError::EmptyBindings);
    }

    #[test]
    fn create_rejects_unknown_tag() {
        let catalog = sample_catalog();
        let err = AcquisitionSession::create(
            "sess-1",
            vec![binding("UNKNOWN", "col")],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap_err();
        assert!(matches!(err, AcquisitionError::UnknownTag(_)));
    }

    #[test]
    fn create_rejects_duplicate_binding() {
        let catalog = sample_catalog();
        let bindings = vec![binding("PT-1101", "col_a"), binding("PT-1101", "col_b")];
        let err =
            AcquisitionSession::create("sess-1", bindings, SamplingPolicy::default(), &catalog)
                .unwrap_err();
        assert!(matches!(err, AcquisitionError::DuplicateBinding(_)));
    }

    #[test]
    fn start_from_created_ok() {
        let catalog = sample_catalog();
        let mut session = AcquisitionSession::create(
            "sess-1",
            vec![binding("PT-1101", "col")],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap();
        let event = session.start().unwrap();
        assert_eq!(event.session, "sess-1");
        assert_eq!(event.tag_count, 1);
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn start_again_fails() {
        let catalog = sample_catalog();
        let mut session = AcquisitionSession::create(
            "sess-1",
            vec![binding("PT-1101", "col")],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap();
        session.start().unwrap();
        let err = session.start().unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidStateTransition { .. }
        ));
    }

    #[test]
    fn stop_from_running_ok() {
        let catalog = sample_catalog();
        let mut session = AcquisitionSession::create(
            "sess-1",
            vec![binding("PT-1101", "col")],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap();
        session.start().unwrap();
        let event = session.stop(5).unwrap();
        assert_eq!(event.batches_recorded, 5);
        assert_eq!(session.state(), SessionState::Stopped);
    }

    #[test]
    fn stop_from_created_fails() {
        let catalog = sample_catalog();
        let mut session = AcquisitionSession::create(
            "sess-1",
            vec![binding("PT-1101", "col")],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap();
        let err = session.stop(0).unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::InvalidStateTransition { .. }
        ));
    }
}
