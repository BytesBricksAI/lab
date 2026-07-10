//! `SimPlant` Lab shared kernel.
//!
//! Pure-domain value objects for industrial process data: tag identifiers, units,
//! quality flags, engineering ranges, alarm limits, time windows, and measurements.
//! This crate has no `re_*` dependencies and performs no I/O.

mod alarm_limits;
mod error;
mod measurement;
mod quality;
mod range;
mod tag_id;
mod time_window;
mod unit;

pub use alarm_limits::AlarmLimits;
pub use error::{KernelError, Result};
pub use measurement::{Measurement, MeasurementBatch};
pub use quality::Quality;
pub use range::EngineeringRange;
pub use tag_id::TagId;
pub use time_window::TimeWindow;
pub use unit::{Dimension, UnitOfMeasure};
