//! Log a simple undirected graph.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_graph_undirected",
    )
    .spawn()?;

    rec.log(
        "simple",
        &[
            &simplant_lab::GraphNodes::new(["a", "b", "c"])
                .with_positions([(0.0, 100.0), (-100.0, 0.0), (100.0, 0.0)])
                .with_labels(["A", "B", "C"])
                as &dyn simplant_lab::AsComponents,
            &simplant_lab::GraphEdges::new([
                ("a", "b"),
                ("b", "c"),
                ("c", "a"),
            ])
            // Optional: graphs are undirected by default.
            .with_undirected_edges(),
        ],
    )?;

    Ok(())
}
