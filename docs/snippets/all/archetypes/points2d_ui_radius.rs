//! Log some points with ui points & scene unit radii.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_points2d_ui_radius",
    )
    .spawn()?;

    // Two blue points with scene unit radii of 0.1 and 0.3.
    rec.log(
        "scene_units",
        &simplant_lab::Points2D::new([(0.0, 0.0), (0.0, 1.0)])
            // By default, radii are interpreted as world-space units.
            .with_radii([0.1, 0.3])
            .with_colors([simplant_lab::Color::from_rgb(0, 0, 255)]),
    )?;

    // Two red points with ui point radii of 40 and 60.
    // UI points are independent of zooming in Views, but are sensitive to the application UI scaling.
    // For 100% ui scaling, UI points are equal to pixels.
    rec.log(
        "ui_points",
        &simplant_lab::Points2D::new([(1.0, 0.0), (1.0, 1.0)])
            // simplant_lab::Radius::new_ui_points produces a radius that the viewer interprets as given in ui points.
            .with_radii([
                simplant_lab::Radius::new_ui_points(40.0),
                simplant_lab::Radius::new_ui_points(60.0),
            ])
            .with_colors([simplant_lab::Color::from_rgb(255, 0, 0)]),
    )?;

    // TODO(#5521): log VisualBounds2D

    Ok(())
}
