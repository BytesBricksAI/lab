//! Update a transform over time.
//!
//! See also the `transform3d_column_updates` example, which achieves the same thing in a single operation.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_transform3d_row_updates",
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

    for t in 0..100 {
        rec.set_time_sequence("tick", t + 1);
        rec.log(
            "box",
            &simplant_lab::Transform3D::default()
                .with_translation([0.0, 0.0, t as f32 / 10.0])
                .with_rotation(simplant_lab::RotationAxisAngle::new(
                    [0.0, 1.0, 0.0],
                    simplant_lab::Angle::from_radians(truncated_radians(
                        (t * 4) as f32,
                    )),
                )),
        )?;
    }

    Ok(())
}

fn truncated_radians(deg: f32) -> f32 {
    ((deg.to_radians() * 1000.0) as i32) as f32 / 1000.0
}
