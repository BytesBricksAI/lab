//! Public API surface for `sp_acquisition`.

pub use crate::application::ports::{DataSourcePort, MeasurementSource, RecorderPort};
pub use crate::application::run_session::run_session;
pub use crate::domain::binding::TagBinding;
pub use crate::domain::error::{AcquisitionError, Result};
pub use crate::domain::events::{
    AcquisitionEvent, AcquisitionStarted, AcquisitionStopped, SourceLost,
};
pub use crate::domain::sampling::SamplingPolicy;
pub use crate::domain::session::{AcquisitionSession, SessionState};
pub use crate::infrastructure::config::{AcquisitionProfile, BindingConfig};
