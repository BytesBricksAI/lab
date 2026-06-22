//! Shows integration of Rerun's `TextLog` with the native logging interface.

use simplant_lab::external::log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_text_log_integration",
    )
    .spawn()?;

    // Log a text entry directly:
    rec.log(
        "logs",
        &simplant_lab::TextLog::new("this entry has loglevel TRACE")
            .with_level(simplant_lab::TextLogLevel::TRACE),
    )?;

    // Or log via a logging handler:
    simplant_lab::Logger::new(rec.clone()) // recording streams are ref-counted
        .with_path_prefix("logs/handler")
        // You can also use the standard `RUST_LOG` environment variable!
        .with_filter(simplant_lab::default_log_filter())
        .init()?;
    log::info!(
        "This INFO log got added through the standard logging interface"
    );

    log::logger().flush();

    Ok(())
}
