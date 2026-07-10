//! Logs a toy P&ID (tank → pump → valve) plus five minutes of simulated
//! process data, and ships a blueprint that lays out the SimPlant Lab
//! viewer with the **P&ID view** on the left half and a trend per linked
//! process variable stacked on the right half.
//!
//! ```bash
//! cargo run -p pid_canvas_demo
//! cargo run -p simplant-lab-cli -- pid_canvas_demo.rrd
//! ```

use re_sdk::RecordingStreamBuilder;
use re_sdk::blueprint::{
    Blueprint, ContainerLike, CustomView, Horizontal, TimePanel, TimeSeriesView, Vertical,
};
use re_sdk_types::archetypes::Scalars;
use sp_types::PidSymbol;

const PLANT_TIME: &str = "plant_time";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "pid_canvas_demo.rrd".to_owned());

    // P&ID on the left half, one trend per linked tag stacked on the right half.
    let blueprint = Blueprint::new(
        Horizontal::new(vec![
            ContainerLike::from(CustomView::new("SimPlantPid", "P&ID").with_origin("/pid")),
            ContainerLike::from(Vertical::new(vec![
                ContainerLike::from(
                    TimeSeriesView::new("TK-101 level").with_origin("/tags/TK-101"),
                ),
                ContainerLike::from(
                    TimeSeriesView::new("P-101 pressure").with_origin("/tags/P-101"),
                ),
                ContainerLike::from(
                    TimeSeriesView::new("XV-101 opening").with_origin("/tags/XV-101"),
                ),
            ])),
        ])
        .with_column_shares([0.5, 0.5]),
    )
    .with_time_panel(TimePanel::new().with_timeline(PLANT_TIME));

    let rec = RecordingStreamBuilder::new("rerun_example_pid_canvas_demo")
        .with_blueprint(blueprint)
        .save(&output)?;

    // Static P&ID layout, in diagram coordinates (y down).
    rec.log_static(
        "pid/TK-101",
        &PidSymbol::new([0.0, 0.0], "PT002A")
            .with_label("TK-101")
            .with_half_size([60.0, 80.0])
            .with_linked_tag("tags/TK-101/level"),
    )?;
    rec.log_static(
        "pid/P-101",
        &PidSymbol::new([260.0, 60.0], "PP007A")
            .with_label("P-101")
            .with_half_size([48.0, 48.0])
            .with_linked_tag("tags/P-101/pressure"),
    )?;
    rec.log_static(
        "pid/XV-101",
        &PidSymbol::new([460.0, 60.0], "ND0001")
            .with_label("XV-101")
            .with_half_size([36.0, 36.0])
            .with_linked_tag("tags/XV-101/opening"),
    )?;

    // Five minutes of simulated process data at 1 Hz.
    let t0 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    for step in 0..300i64 {
        rec.set_timestamp_nanos_since_epoch(PLANT_TIME, (t0 + step) * 1_000_000_000);
        let x = step as f64;

        let level = 74.0 + 8.0 * (x * 0.02).sin();
        let pressure = 12.0 + (x * 0.08).sin() + 0.3 * (x * 0.9).sin();
        let opening = 50.0 + 35.0 * (x * 0.05).cos();

        rec.log("tags/TK-101/level", &Scalars::single(level))?;
        rec.log("tags/P-101/pressure", &Scalars::single(pressure))?;
        rec.log("tags/XV-101/opening", &Scalars::single(opening))?;
    }

    rec.flush_blocking()?;
    println!("Wrote {output}");
    println!("Open it with: cargo run -p simplant-lab-cli -- {output}");
    Ok(())
}
