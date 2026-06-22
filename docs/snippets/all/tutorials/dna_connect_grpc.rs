//! The DNA-abacus example, connecting to a separately-running viewer over gRPC.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the viewer running at the default URL.
    let _rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_dna_abacus")
            .connect_grpc()?;

    // … log data as in the spawn-based example …

    Ok(())
}
