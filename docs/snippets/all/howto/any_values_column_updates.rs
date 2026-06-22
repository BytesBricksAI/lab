//! Update custom user-defined values over time, in a single operation.
//!
//! This is semantically equivalent to the `any_values_row_updates` example, albeit much faster.

#![expect(clippy::from_iter_instead_of_collect)]

use std::sync::Arc;

use simplant_lab::{TimeColumn, external::arrow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_any_values_column_updates",
    )
    .spawn()?;

    const STEPS: i64 = 64;

    let times = TimeColumn::new_sequence("step", 0..STEPS);

    let sin = simplant_lab::SerializedComponentBatch::new(
        Arc::new(arrow::array::Float64Array::from_iter(
            (0..STEPS).map(|v| ((v as f64) / 10.0).sin()),
        )),
        simplant_lab::ComponentDescriptor::partial("sin"),
    );

    let cos = simplant_lab::SerializedComponentBatch::new(
        Arc::new(arrow::array::Float64Array::from_iter(
            (0..STEPS).map(|v| ((v as f64) / 10.0).cos()),
        )),
        simplant_lab::ComponentDescriptor::partial("cos"),
    );

    rec.send_columns(
        "/",
        [times],
        [
            sin.partitioned(std::iter::repeat_n(1, STEPS as _))?,
            cos.partitioned(std::iter::repeat_n(1, STEPS as _))?,
        ],
    )?;

    Ok(())
}
