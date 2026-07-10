//! Log a batch of oriented bounding boxes.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_box3d_batch")
            .spawn()?;

    rec.log(
        "batch",
        &simplant_lab::Boxes3D::from_centers_and_half_sizes(
            [(2.0, 0.0, 0.0), (-2.0, 0.0, 0.0), (0.0, 0.0, 2.0)],
            [(2.0, 2.0, 1.0), (1.0, 1.0, 0.5), (2.0, 0.5, 1.0)],
        )
        .with_quaternions([
            simplant_lab::Quaternion::IDENTITY,
            simplant_lab::Quaternion::from_xyzw([0.0, 0.0, 0.382683, 0.923880]), // 45 degrees around Z
        ])
        .with_radii([0.025])
        .with_colors([
            simplant_lab::Color::from_rgb(255, 0, 0),
            simplant_lab::Color::from_rgb(0, 255, 0),
            simplant_lab::Color::from_rgb(0, 0, 255),
        ])
        .with_fill_mode(simplant_lab::FillMode::Solid)
        .with_labels(["red", "green", "blue"]),
    )?;

    Ok(())
}
