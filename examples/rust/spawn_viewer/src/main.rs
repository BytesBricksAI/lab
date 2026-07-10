//! Spawn a new Rerun Viewer process ready to listen for gRPC connections.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simplant_lab::spawn(&simplant_lab::SpawnOptions::default())?;
    Ok(())
}
