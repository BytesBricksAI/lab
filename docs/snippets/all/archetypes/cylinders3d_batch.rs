//! Log a batch of cylinders.

use simplant_lab::external::glam::vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_cylinders3d_batch",
    )
    .spawn()?;

    rec.log(
        "cylinders",
        &simplant_lab::Cylinders3D::from_lengths_and_radii(
            [0.0, 2.0, 4.0, 6.0, 8.0],
            [1.0, 0.5, 0.5, 0.5, 1.0],
        )
        .with_colors([
            simplant_lab::Color::from_rgb(255, 0, 0),
            simplant_lab::Color::from_rgb(188, 188, 0),
            simplant_lab::Color::from_rgb(0, 255, 0),
            simplant_lab::Color::from_rgb(0, 188, 188),
            simplant_lab::Color::from_rgb(0, 0, 255),
        ])
        .with_centers([
            vec3(0., 0., 0.),
            vec3(2., 0., 0.),
            vec3(4., 0., 0.),
            vec3(6., 0., 0.),
            vec3(8., 0., 0.),
        ])
        .with_rotation_axis_angles((0..5).map(|i| {
            simplant_lab::RotationAxisAngle::new(
                [1.0, 0.0, 0.0],
                simplant_lab::Angle::from_degrees(i as f32 * -22.5),
            )
        })),
    )?;

    Ok(())
}
