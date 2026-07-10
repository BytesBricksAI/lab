//! Log some very simple 2D ellipses.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_ellipses2d")
            .spawn()?;

    rec.log(
        "simple",
        &simplant_lab::Ellipses2D::from_centers_and_half_sizes(
            [(0.0, 0.0)],
            [(2.0, 1.0)],
        ),
    )?;

    Ok(())
}
