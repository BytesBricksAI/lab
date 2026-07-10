//! Log some very simple points.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_points3d")
            .spawn()?;

    rec.log(
        "points",
        &simplant_lab::Points3D::new([(0.0, 0.0, 0.0), (1.0, 1.0, 1.0)]),
    )?;

    Ok(())
}
