//! Log a scalar over time.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new("rerun_example_scalar")
        .spawn()?;

    // Log the data on a timeline called "step".
    for step in 0..64 {
        rec.set_time_sequence("step", step);
        rec.log(
            "scalar",
            &simplant_lab::Scalars::single((step as f64 / 10.0).sin()),
        )?;
    }

    Ok(())
}
