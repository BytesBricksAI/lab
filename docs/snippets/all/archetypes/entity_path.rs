//! Example of different ways of constructing an entity path.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_entity_path")
            .spawn()?;

    rec.log(
        r"world/42/escaped\ string\!",
        &simplant_lab::TextDocument::new(
            "This entity path was escaped manually",
        ),
    )?;
    rec.log(
        simplant_lab::entity_path!["world", 42, "unescaped string!"],
        &simplant_lab::TextDocument::new(
            "This entity path was provided as a list of unescaped strings",
        ),
    )?;

    Ok(())
}
