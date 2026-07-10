//! Log a PNG image

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_encoded_image",
    )
    .spawn()?;

    let image = include_bytes!("ferris.png");

    rec.log(
        "image",
        &simplant_lab::EncodedImage::from_file_contents(image.to_vec()),
    )?;

    Ok(())
}
