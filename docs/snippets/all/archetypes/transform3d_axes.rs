//! Log different transforms with visualized coordinates axes.

use simplant_lab::AsComponents;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_transform3d_axes",
    )
    .spawn()?;

    rec.set_time_sequence("step", 0);

    rec.log(
        "base",
        &[
            &simplant_lab::Transform3D::new() as &dyn AsComponents,
            &simplant_lab::TransformAxes3D::new(1.0),
        ],
    )?;

    for deg in 0..360 {
        rec.set_time_sequence("step", deg);
        rec.log(
            "base/rotated",
            &[
                &simplant_lab::Transform3D::new().with_rotation(
                    simplant_lab::RotationAxisAngle::new(
                        [1.0, 1.0, 1.0],
                        simplant_lab::Angle::from_degrees(deg as f32),
                    ),
                ) as &dyn AsComponents,
                &simplant_lab::TransformAxes3D::new(0.5),
            ],
        )?;
        rec.log(
            "base/rotated/translated",
            &[
                &simplant_lab::Transform3D::new()
                    .with_translation([2.0, 0.0, 0.0])
                    as &dyn AsComponents,
                &simplant_lab::TransformAxes3D::new(0.5),
            ],
        )?;
    }

    Ok(())
}
