//! Log a `TextLog`

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_text_log")
            .spawn()?;

    rec.log(
        "log",
        &simplant_lab::TextLog::new("Application started.").with_level("INFO"),
    )?;

    Ok(())
}
