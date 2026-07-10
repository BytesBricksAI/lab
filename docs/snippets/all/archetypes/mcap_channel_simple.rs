//! Log a simple MCAP channel definition.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_mcap_channel")
            .spawn()?;

    rec.log(
        "mcap/channels/camera",
        &simplant_lab::McapChannel::new(1, "/camera/image", "cdr")
            .with_metadata([("frame_id", "camera_link"), ("encoding", "bgr8")]),
    )?;

    Ok(())
}
