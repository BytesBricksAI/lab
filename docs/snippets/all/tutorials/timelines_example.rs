for frame in read_sensor_frames() {
    rec.set_time_sequence("frame_idx", frame.idx);
    rec.set_timestamp_secs_since_epoch("sensor_time", frame.timestamp);

    rec.log("sensor/points", simplant_lab::Points3D::new(&frame.points))?;
}
