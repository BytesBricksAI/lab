//! Log some very simple geospatial point.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_geo_points")
            .spawn()?;

    rec.log(
        "rerun_hq",
        &simplant_lab::GeoPoints::from_lat_lon([(59.319221, 18.075631)])
            .with_radii([simplant_lab::Radius::new_ui_points(10.0)])
            .with_colors([simplant_lab::Color::from_rgb(255, 0, 0)]),
    )?;

    Ok(())
}
