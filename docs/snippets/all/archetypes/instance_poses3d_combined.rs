//! Log a simple 3D box with a regular & instance pose transform.

use simplant_lab::{
    demo_util::grid,
    external::{anyhow, glam},
};

fn main() -> anyhow::Result<()> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_instance_pose3d_combined",
    )
    .spawn()?;

    rec.set_time_sequence("frame", 0);

    // Log a box and points further down in the hierarchy.
    rec.log(
        "world/box",
        &simplant_lab::Boxes3D::from_half_sizes([[1.0, 1.0, 1.0]]),
    )?;
    rec.log(
        "world/box/points",
        &simplant_lab::Points3D::new(grid(
            glam::Vec3::splat(-10.0),
            glam::Vec3::splat(10.0),
            10,
        )),
    )?;

    for i in 0..180 {
        rec.set_time_sequence("frame", i);

        // Log a regular transform which affects both the box and the points.
        rec.log(
            "world/box",
            &simplant_lab::Transform3D::from_rotation(
                simplant_lab::RotationAxisAngle {
                    axis: [0.0, 0.0, 1.0].into(),
                    angle: simplant_lab::Angle::from_degrees(i as f32 * 2.0),
                },
            ),
        )?;

        // Log an instance pose which affects only the box.
        let translation = [0.0, 0.0, (i as f32 * 0.1 - 5.0).abs() - 5.0];
        rec.log(
            "world/box",
            &simplant_lab::InstancePoses3D::new()
                .with_translations([translation]),
        )?;
    }

    Ok(())
}
