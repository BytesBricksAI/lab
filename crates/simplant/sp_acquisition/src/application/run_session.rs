//! Run acquisition session use case.

use sp_asset_model::AssetCatalog;

use crate::application::ports::{DataSourcePort, RecorderPort};
use crate::domain::error::{AcquisitionError, Result};
use crate::domain::events::{AcquisitionEvent, SourceLost};
use crate::domain::session::AcquisitionSession;

/// Orchestrates an acquisition session end to end (used by the `acquire` CLI subcommand).
///
/// 1) starts the session, 2) records static metadata of bound tags from the catalog,
/// 3) records `AcquisitionStarted`, 4) pumps batches from the source into the recorder,
/// 5) records `AcquisitionStopped`. Returns the number of batches recorded.
pub fn run_session(
    session: &mut AcquisitionSession,
    catalog: &AssetCatalog,
    source: &dyn DataSourcePort,
    recorder: &dyn RecorderPort,
) -> Result<u64> {
    let started = session.start()?;

    for binding in session.bindings() {
        let tag = catalog
            .tag(binding.tag())
            .ok_or_else(|| AcquisitionError::UnknownTag(binding.tag().as_str().to_owned()))?;
        recorder.record_tag_metadata(tag)?;
    }

    recorder.record_event(&AcquisitionEvent::Started(started))?;

    let mut measurement_source = source.subscribe(session.bindings(), &session.policy())?;

    let mut batches_recorded: u64 = 0;
    loop {
        match measurement_source.next_batch() {
            Ok(Some(batch)) => {
                recorder.record_batch(&batch)?;
                batches_recorded += 1;
            }
            Ok(None) => break,
            Err(err) => {
                recorder
                    .record_event(&AcquisitionEvent::SourceLost(SourceLost {
                        session: session.id().to_owned(),
                        reason: err.to_string(),
                    }))
                    .ok();
                return Err(err);
            }
        }
    }

    let stopped = session.stop(batches_recorded)?;
    recorder.record_event(&AcquisitionEvent::Stopped(stopped))?;

    Ok(batches_recorded)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use sp_asset_model::{
        AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
        FacilityId, Tag, TagSpec, UnitId,
    };
    use sp_kernel::{MeasurementBatch, TagId, UnitOfMeasure};

    use super::*;
    use crate::application::ports::{DataSourcePort, MeasurementSource, RecorderPort};
    use crate::domain::binding::TagBinding;
    use crate::domain::events::AcquisitionEvent;
    use crate::domain::sampling::SamplingPolicy;

    struct FakeMeasurementSource {
        batches: Vec<MeasurementBatch>,
        index: usize,
    }

    impl MeasurementSource for FakeMeasurementSource {
        fn next_batch(&mut self) -> Result<Option<MeasurementBatch>> {
            if self.index < self.batches.len() {
                let batch = self.batches[self.index].clone();
                self.index += 1;
                Ok(Some(batch))
            } else {
                Ok(None)
            }
        }
    }

    struct FakeSource {
        batches: Vec<MeasurementBatch>,
    }

    impl DataSourcePort for FakeSource {
        fn subscribe(
            &self,
            _bindings: &[TagBinding],
            _policy: &SamplingPolicy,
        ) -> Result<Box<dyn MeasurementSource>> {
            Ok(Box::new(FakeMeasurementSource {
                batches: self.batches.clone(),
                index: 0,
            }))
        }
    }

    #[derive(Default)]
    struct RecorderCounts {
        metadata: usize,
        batches: usize,
        started: usize,
        stopped: usize,
    }

    struct FakeRecorder {
        counts: RefCell<RecorderCounts>,
    }

    impl RecorderPort for FakeRecorder {
        fn record_batch(&self, _batch: &MeasurementBatch) -> Result<()> {
            self.counts.borrow_mut().batches += 1;
            Ok(())
        }

        fn record_tag_metadata(&self, _tag: &Tag) -> Result<()> {
            self.counts.borrow_mut().metadata += 1;
            Ok(())
        }

        fn record_event(&self, event: &AcquisitionEvent) -> Result<()> {
            let mut counts = self.counts.borrow_mut();
            match event {
                AcquisitionEvent::Started(_) => counts.started += 1,
                AcquisitionEvent::Stopped(_) => counts.stopped += 1,
                AcquisitionEvent::SourceLost(_) => {}
            }
            Ok(())
        }
    }

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
            range: sp_kernel::EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(spec).unwrap();

        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    fn sample_batch(tag: &str) -> MeasurementBatch {
        MeasurementBatch::new(TagId::new(tag).unwrap(), vec![])
    }

    #[test]
    fn run_session_records_batches_and_events() {
        let catalog = sample_catalog();
        let binding = TagBinding::new(TagId::new("PT-1101").unwrap(), "col").unwrap();
        let mut session = AcquisitionSession::create(
            "sess-1",
            vec![binding],
            SamplingPolicy::default(),
            &catalog,
        )
        .unwrap();

        let tag = "PT-1101";
        let batches = vec![sample_batch(tag), sample_batch(tag), sample_batch(tag)];

        let source = FakeSource {
            batches: batches.clone(),
        };
        let recorder = FakeRecorder {
            counts: RefCell::new(RecorderCounts::default()),
        };

        let count = run_session(&mut session, &catalog, &source, &recorder).unwrap();
        assert_eq!(count, 3);

        let counts = recorder.counts.borrow();
        assert_eq!(counts.metadata, 1);
        assert_eq!(counts.batches, 3);
        assert_eq!(counts.started, 1);
        assert_eq!(counts.stopped, 1);
    }
}
