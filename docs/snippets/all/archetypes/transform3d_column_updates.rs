//! Update a transform over time, in a single operation.
//!
//! This is semantically equivalent to the `transform3d_row_updates` example, albeit much faster.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_transform3d_column_updates",
    )
    .spawn()?;

    rec.set_time_sequence("tick", 0);
    rec.log(
        "box",
        &[
            &simplant_lab::Boxes3D::from_half_sizes([(4.0, 2.0, 1.0)])
                .with_fill_mode(simplant_lab::FillMode::Solid)
                as &dyn simplant_lab::AsComponents,
            &simplant_lab::TransformAxes3D::new(10.0),
        ],
    )?;

    let translations = (0..100).map(|t| [0.0, 0.0, t as f32 / 10.0]);
    let rotations =
        (0..100)
            .map(|t| truncated_radians((t * 4) as f32))
            .map(|rad| {
                simplant_lab::RotationAxisAngle::new(
                    [0.0, 1.0, 0.0],
                    simplant_lab::Angle::from_radians(rad),
                )
            });

    let ticks = simplant_lab::TimeColumn::new_sequence("tick", 1..101);
    rec.send_columns(
        "box",
        [ticks],
        simplant_lab::Transform3D::default()
            .with_many_translation(translations)
            .with_many_rotation_axis_angle(rotations)
            .columns_of_unit_batches()?,
    )?;

    Ok(())
}

fn truncated_radians(deg: f32) -> f32 {
    ((deg.to_radians() * 1000.0) as i32) as f32 / 1000.0
}
