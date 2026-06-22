//! Log a simple occupancy grid map.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec =
        simplant_lab::RecordingStreamBuilder::new("rerun_example_grid_map")
            .spawn()?;

    let width: usize = 64;
    let height: usize = 64;
    let cell_size: f32 = 0.1;

    // Create a synthetic image with ROS `nav_msgs/OccupancyGrid` cell value conventions:
    // -1 (255) unknown, 0 free, 100 occupied.
    let mut grid = vec![255u8; width * height];
    for y in 8..56 {
        for x in 8..56 {
            grid[y * width + x] = 0;
        }
    }
    for y in 20..44 {
        for x in 20..44 {
            grid[y * width + x] = 100;
        }
    }

    rec.log(
        "world/map",
        &simplant_lab::GridMap::new(
            grid,
            simplant_lab::components::ImageFormat::from_color_model(
                [width as u32, height as u32],
                simplant_lab::ColorModel::L,
                simplant_lab::ChannelDatatype::U8,
            ),
            cell_size,
        )
        .with_translation([
            -(width as f32) * cell_size / 2.0,
            -(height as f32) * cell_size / 2.0,
            0.0,
        ])
        .with_colormap(simplant_lab::components::Colormap::RvizMap),
    )?;

    Ok(())
}
