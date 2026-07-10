//! Logs a simple transform hierarchy with named frames.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_transform3d_hierarchy_named_frames",
    )
    .spawn()?;

    // Define entities with explicit coordinate frames.
    rec.log(
        "sun",
        &[
            &simplant_lab::Ellipsoids3D::from_half_sizes([[1.0, 1.0, 1.0]])
                .with_colors([simplant_lab::Color::from_rgb(255, 200, 10)])
                .with_fill_mode(simplant_lab::FillMode::Solid)
                as &dyn simplant_lab::AsComponents,
            &simplant_lab::CoordinateFrame::new("sun_frame"),
        ],
    )?;

    rec.log(
        "planet",
        &[
            &simplant_lab::Ellipsoids3D::from_half_sizes([[0.4, 0.4, 0.4]])
                .with_colors([simplant_lab::Color::from_rgb(40, 80, 200)])
                .with_fill_mode(simplant_lab::FillMode::Solid)
                as &dyn simplant_lab::AsComponents,
            &simplant_lab::CoordinateFrame::new("planet_frame"),
        ],
    )?;

    rec.log(
        "moon",
        &[
            &simplant_lab::Ellipsoids3D::from_half_sizes([[0.15, 0.15, 0.15]])
                .with_colors([simplant_lab::Color::from_rgb(180, 180, 180)])
                .with_fill_mode(simplant_lab::FillMode::Solid)
                as &dyn simplant_lab::AsComponents,
            &simplant_lab::CoordinateFrame::new("moon_frame"),
        ],
    )?;

    // Define explicit frame relationships.
    rec.log(
        "planet_transform",
        &simplant_lab::Transform3D::from_translation([6.0, 0.0, 0.0])
            .with_child_frame("planet_frame")
            .with_parent_frame("sun_frame"),
    )?;

    rec.log(
        "moon_transform",
        &simplant_lab::Transform3D::from_translation([3.0, 0.0, 0.0])
            .with_child_frame("moon_frame")
            .with_parent_frame("planet_frame"),
    )?;

    // Connect the viewer to the sun's coordinate frame.
    // This is only needed in the absence of blueprints since a default view will typically be created at `/`.
    rec.log_static("/", &simplant_lab::CoordinateFrame::new("sun_frame"))?;

    Ok(())
}
