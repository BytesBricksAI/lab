//! Log arbitrary archetype data.

use std::sync::Arc;

use simplant_lab::external::arrow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_dynamic_archetype",
    )
    .spawn()?;

    let new_archetype = simplant_lab::DynamicArchetype::new("MyArchetype")
        // Using arbitrary Arrow data.
        .with_component_from_data(
            "homepage",
            Arc::new(arrow::array::StringArray::from(vec![
                "https://www.rerun.io",
            ])),
        )
        .with_component_from_data(
            "repository",
            Arc::new(arrow::array::StringArray::from(vec![
                "https://github.com/rerun-io/rerun",
            ])),
        )
        // Using Rerun's builtin components.
        .with_component::<simplant_lab::components::Scalar>(
            "confidence",
            [1.2, 3.4, 5.6],
        )
        .with_component::<simplant_lab::components::Text>(
            "description",
            vec!["Bla bla bla…"],
        );

    rec.log("new_archetype", &new_archetype)?;

    Ok(())
}
