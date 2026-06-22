//! Log a `StateChange`

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_state_change")
            .spawn()?;

    rec.set_time_sequence("step", 0);
    rec.log("door", &simplant_lab::StateChange::new().with_state("open"))?;

    rec.set_time_sequence("step", 1);
    rec.log(
        "door",
        &simplant_lab::StateChange::new().with_state("closed"),
    )?;

    rec.set_time_sequence("step", 2);
    rec.log("door", &simplant_lab::StateChange::new().with_state("open"))?;

    Ok(())
}
