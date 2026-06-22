//! Log different transforms between three arrows.

use std::f32::consts::TAU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_transform3d")
            .spawn()?;

    let arrow = simplant_lab::Arrows3D::from_vectors([(0.0, 1.0, 0.0)])
        .with_origins([(0.0, 0.0, 0.0)]);

    rec.log("base", &arrow)?;

    rec.log(
        "base/translated",
        &simplant_lab::Transform3D::from_translation([1.0, 0.0, 0.0]),
    )?;

    rec.log("base/translated", &arrow)?;

    rec.log(
        "base/rotated_scaled",
        &simplant_lab::Transform3D::from_rotation_scale(
            simplant_lab::RotationAxisAngle::new(
                [0.0, 0.0, 1.0],
                simplant_lab::Angle::from_radians(TAU / 8.0),
            ),
            simplant_lab::Scale3D::from(2.0),
        ),
    )?;

    rec.log("base/rotated_scaled", &arrow)?;

    Ok(())
}
