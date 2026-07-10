//! Log a pinhole and a random image.

use ndarray::{Array, ShapeBuilder as _};
use rand::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_pinhole")
            .spawn()?;

    let mut image = Array::<u8, _>::default((3, 3, 3).f());
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    image.map_inplace(|x| *x = rng.random());

    rec.log(
        "world/image",
        &simplant_lab::Pinhole::from_focal_length_and_resolution(
            [3., 3.],
            [3., 3.],
        ),
    )?;
    rec.log(
        "world/image",
        &simplant_lab::Image::from_color_model_and_tensor(
            simplant_lab::ColorModel::RGB,
            image,
        )?,
    )?;

    Ok(())
}
