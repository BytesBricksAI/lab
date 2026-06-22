//! Continuously print the connection status of a pending gRPC connection.
//!
//! This feature is experimental and may change in future releases.

#![expect(clippy::disallowed_methods)] // We forbid naked `send` calls in core Rerun, but they are fine in snippets

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rec = simplant_lab::RecordingStreamBuilder::new(
        "rerun_example_check_connection_status",
    )
    .connect_grpc()?;

    let (tx, rx) = crossbeam::channel::bounded(1);

    loop {
        let tx = tx.clone();
        rec.inspect_sink(move |sink| {
            let grpc_sink = (sink as &dyn std::any::Any)
                .downcast_ref::<simplant_lab::sink::GrpcSink>()
                .expect("Expected a GrpcSink");
            tx.send(grpc_sink.status()).ok();
        });

        if let Ok(status) = rx.recv_timeout(std::time::Duration::from_secs(1)) {
            println!("Connection status: {status:?}");

            if matches!(
                status,
                simplant_lab::sink::GrpcSinkConnectionState::Disconnected(_)
            ) {
                println!("Connection lost, exiting");
                break;
            }
        } else {
            println!("No connection status received for 1s, exiting");
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
