//! Log a simple 3D asset.

use simplant_lab::external::anyhow;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let Some(path) = args.get(1) else {
        anyhow::bail!("Usage: {} <path_to_asset.[gltf|glb|obj|stl]>", args[0]);
    };

    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_asset3d")
            .spawn()?;

    rec.log_static("world", &simplant_lab::ViewCoordinates::RIGHT_HAND_Z_UP())?; // Set an up-axis
    rec.log("world/asset", &simplant_lab::Asset3D::from_file_path(path)?)?;

    Ok(())
}
