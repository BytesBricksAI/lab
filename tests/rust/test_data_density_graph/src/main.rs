//! Visual test for timeline density graphs.
//!
//! ```
//! cargo run -p test_data_density_graph
//! ```

use simplant_lab::RecordingStream;
use simplant_lab::external::re_log_types::NonMinI64;
use simplant_lab::external::{re_chunk_store, re_log};
use simplant_lab::time::TimeInt;

fn main() -> anyhow::Result<()> {
    re_log::setup_logging();

    let rec = simplant_lab::RecordingStreamBuilder::new("rerun_example_test_data_density_graph")
        .spawn_opts(&simplant_lab::SpawnOptions {
            wait_for_bind: true,
            extra_env: {
                use re_chunk_store::ChunkStoreConfig as C;
                vec![
                    (C::ENV_CHUNK_MAX_BYTES.into(), "0".into()),
                    (C::ENV_CHUNK_MAX_ROWS.into(), "0".into()),
                    (C::ENV_CHUNK_MAX_ROWS_IF_UNSORTED.into(), "0".into()),
                ]
            },
            ..Default::default()
        })?;

    run(&rec)
}

fn run(rec: &RecordingStream) -> anyhow::Result<()> {
    const DESCRIPTION: &str = "\
    Logs different kinds of chunks to exercise different code paths for the density graphs in the time panel:

    - Many small chunks
    - A few large sorted chunks
    - A few large unsorted chunks
    ";
    rec.log_static(
        "description",
        &simplant_lab::TextDocument::from_markdown(DESCRIPTION),
    )?;

    let entities = [
        ("/small", 100, 100, true, 0),
        ("/large", 5, 2000, true, 0),
        ("/large-unsorted", 5, 2000, false, 0),
        ("/gap", 2, 5000, true, 500000),
        ("/over-threshold", 1, 100000, true, 5000000),
    ];

    for (entity_path, num_chunks, num_rows_per_chunk, sorted, time_start_ms) in entities {
        log(
            rec,
            entity_path,
            num_chunks,
            num_rows_per_chunk,
            sorted,
            time_start_ms,
        )?;
    }

    Ok(())
}

fn log(
    rec: &RecordingStream,
    entity_path: &str,
    num_chunks: i64,
    num_rows_per_chunk: i64,
    sorted: bool,
    time_start_ms: i64,
) -> anyhow::Result<()> {
    let entity_path = simplant_lab::EntityPath::parse_strict(entity_path)?;

    // log points
    rec.send_chunk(
        simplant_lab::log::Chunk::builder(entity_path.clone())
            .with_archetype(
                simplant_lab::log::RowId::new(),
                [
                    (
                        simplant_lab::Timeline::log_time(),
                        simplant_lab::time::TimeInt::from_millis(NonMinI64::ZERO),
                    ),
                    (simplant_lab::Timeline::log_tick(), TimeInt::ZERO),
                ],
                &simplant_lab::Points3D::new(simplant_lab::demo_util::grid(
                    (-10.0, -10.0, -10.0).into(),
                    (10.0, 10.0, 10.0).into(),
                    10,
                )),
            )
            .build()?,
    );

    let mut time = time_start_ms;
    for _ in 0..num_chunks {
        let mut log_times = vec![];
        for _ in 0..num_rows_per_chunk {
            time += 1;
            log_times.push(time);
        }
        time += 100;
        log_times.push(time);

        if !sorted {
            let mut rng = rand::rng();
            use rand::seq::SliceRandom as _;
            log_times.shuffle(&mut rng);
        }

        let components = (0..num_rows_per_chunk).map(|i| {
            let angle_deg = i as f32 % 360.0;
            simplant_lab::Transform3D::from_rotation(simplant_lab::Rotation3D::AxisAngle(
                (
                    (0.0, 0.0, 1.0),
                    simplant_lab::Angle::from_degrees(angle_deg),
                )
                    .into(),
            ))
        });

        let mut chunk = simplant_lab::log::Chunk::builder(entity_path.clone());
        for (time, component) in log_times.iter().zip(components) {
            chunk = chunk.with_archetype(
                simplant_lab::log::RowId::new(),
                [(
                    simplant_lab::Timeline::log_time(),
                    simplant_lab::time::TimeInt::from_millis(
                        (*time).try_into().unwrap_or_default(),
                    ),
                )],
                &component,
            );
        }
        rec.send_chunk(chunk.build()?);
    }

    Ok(())
}
