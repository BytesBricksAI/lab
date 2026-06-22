use simplant_lab::external::re_log;

const W: usize = 200;
const H: usize = 200;
const MIN_Z: usize = 10;
const MAX_Z: usize = 20;

struct Input {
    centers: Vec<glam::Vec3>,
}

fn prepare() -> Input {
    let mut centers = vec![];
    for x in 0..W {
        for y in 0..H {
            let height = emath::remap(
                ((x + y) as f32 * 0.1).sin(),
                -1.0..=1.0,
                MIN_Z as f32..=MAX_Z as f32,
            );
            for z in 0..height.round() as usize {
                centers.push(glam::Vec3::new(x as f32, y as f32, z as f32));
            }
        }
    }

    re_log::info!("Logging {} boxes", centers.len());

    Input { centers }
}

fn execute(rec: &simplant_lab::RecordingStream, input: Input) -> anyhow::Result<()> {
    re_tracing::profile_function!();

    let Input { centers } = input;

    rec.log(
        "large_batch",
        &simplant_lab::Boxes3D::update_fields()
            .with_half_sizes([simplant_lab::HalfSize3D::new(0.5, 0.5, 0.5)])
            .with_centers(centers)
            .with_fill_mode(simplant_lab::FillMode::Solid),
    )?;
    Ok(())
}

/// Emulate a voxel occupancy grid
pub fn run(rec: &simplant_lab::RecordingStream) -> anyhow::Result<()> {
    re_tracing::profile_function!();
    let input = std::hint::black_box(prepare());
    execute(rec, input)
}
