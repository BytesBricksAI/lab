//! Demonstrates how to use `RecordingId`s to build a single recording from multiple processes.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new("rerun_example_shared_recording")
        .recording_id("my_shared_recording")
        .spawn()?;

    rec.log(
        "updates",
        &simplant_lab::TextLog::new(format!("hello from process #{}", std::process::id())),
    )?;

    Ok(())
}
