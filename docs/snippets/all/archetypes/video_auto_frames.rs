//! Log a video asset using automatically determined frame references.

use simplant_lab::external::anyhow;

fn main() -> anyhow::Result<()> {
    let args = _args;
    let Some(path) = args.get(1) else {
        // TODO(#7354): Only mp4 is supported for now.
        anyhow::bail!("Usage: {} <path_to_video.[mp4]>", args[0]);
    };

    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_asset_video_auto_frames",
    )
    .spawn()?;

    // Log video asset which is referred to by frame references.
    let video_asset = simplant_lab::AssetVideo::from_file_path(path)?;
    rec.log_static("video", &video_asset)?;

    // Send automatically determined video frame timestamps.
    let frame_timestamps_nanos = video_asset.read_frame_timestamps_nanos()?;
    let video_timestamps_nanos = frame_timestamps_nanos
        .iter()
        .copied()
        .map(simplant_lab::components::VideoTimestamp::from_nanos)
        .collect::<Vec<_>>();
    let time_column = simplant_lab::TimeColumn::new_duration_nanos(
        "video_time",
        // Note timeline values don't have to be the same as the video timestamps.
        frame_timestamps_nanos,
    );

    rec.send_columns(
        "video",
        [time_column],
        simplant_lab::VideoFrameReference::update_fields()
            .with_many_timestamp(video_timestamps_nanos)
            .columns_of_unit_batches()?,
    )?;

    Ok(())
}
