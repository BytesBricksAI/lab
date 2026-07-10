//! Log a simple geospatial line string.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_geo_line_strings",
    )
    .spawn()?;

    rec.log(
        "colorado",
        &simplant_lab::GeoLineStrings::from_lat_lon([[
            [41.0000, -109.0452],
            [41.0000, -102.0415],
            [36.9931, -102.0415],
            [36.9931, -109.0452],
            [41.0000, -109.0452],
        ]])
        .with_radii([simplant_lab::Radius::new_ui_points(2.0)])
        .with_colors([simplant_lab::Color::from_rgb(0, 0, 255)]),
    )?;

    Ok(())
}
