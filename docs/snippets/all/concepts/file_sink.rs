//! Create and set a file sink.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_file_sink")
            .buffered()?;

    rec.set_sink(Box::new(simplant_lab::sink::FileSink::new(
        "recording.rrd",
    )?));

    Ok(())
}
