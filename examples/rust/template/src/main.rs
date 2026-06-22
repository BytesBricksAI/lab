//! Example template.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new("rerun_example_my_example_name").spawn()?;

    // … example code
    _ = rec;

    Ok(())
}
