//! Create and log an image

use ndarray::{Array, ShapeBuilder as _, s};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new("rerun_example_image")
        .spawn()?;

    let mut image = Array::<u8, _>::zeros((200, 300, 3).f());
    image.slice_mut(s![.., .., 0]).fill(255);
    image.slice_mut(s![50..150, 50..150, 0]).fill(0);
    image.slice_mut(s![50..150, 50..150, 1]).fill(255);

    rec.log(
        "image",
        &simplant_lab::Image::from_color_model_and_tensor(
            simplant_lab::ColorModel::RGB,
            image,
        )?,
    )?;

    Ok(())
}
