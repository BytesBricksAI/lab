//! Modbus TCP live polling source.

use std::net::SocketAddr;
use std::str::FromStr as _;

use jiff::Timestamp;
use sp_acquisition::{
    AcquisitionError, DataSourcePort, MeasurementSource, Result, SamplingPolicy, TagBinding,
};
use sp_kernel::{Measurement, MeasurementBatch, Quality, TagId};
use tokio_modbus::ExceptionCode;
use tokio_modbus::client::{Context, tcp};
use tokio_modbus::prelude::*;

use crate::address::{ModbusPoint, RegisterKind, map_register, parse_modbus_address};

/// Read-only Modbus TCP data source.
pub struct ModbusTcpSource {
    addr: SocketAddr,
}

impl ModbusTcpSource {
    /// Creates a source targeting `addr`.
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Parses `host:port` and creates a source.
    pub fn parse_addr(host_port: &str) -> Result<Self> {
        let addr = SocketAddr::from_str(host_port)
            .map_err(|err| AcquisitionError::Source(format!("invalid socket address: {err}")))?;
        Ok(Self::new(addr))
    }
}

impl std::str::FromStr for ModbusTcpSource {
    type Err = AcquisitionError;

    fn from_str(host_port: &str) -> Result<Self> {
        Self::parse_addr(host_port)
    }
}

struct ModbusPollSource {
    runtime: tokio::runtime::Runtime,
    client: Context,
    points: Vec<(TagId, ModbusPoint)>,
    next_index: usize,
}

impl ModbusPollSource {
    fn read_raw_register(&mut self, point: &ModbusPoint) -> Result<u16> {
        let register = point.register();
        let kind = point.kind();

        self.runtime.block_on(async {
            let words_result = match kind {
                RegisterKind::Holding => self.client.read_holding_registers(register, 1).await,
                RegisterKind::Input => self.client.read_input_registers(register, 1).await,
            };

            let words = words_result.map_err(|err| {
                AcquisitionError::Source(format!("modbus transport error: {err}"))
            })?;

            let words = words.map_err(|code: ExceptionCode| {
                AcquisitionError::Source(format!("modbus exception: {code:?}"))
            })?;

            words.into_iter().next().ok_or_else(|| {
                AcquisitionError::Source(format!(
                    "modbus read returned no words for register {register}"
                ))
            })
        })
    }
}

impl MeasurementSource for ModbusPollSource {
    /// Polls the next bound tag and returns one sample batch.
    ///
    /// Unlike replay sources, a live Modbus poll never exhausts: this method always
    /// returns `Ok(Some(batch))` on success. The caller controls poll rate by how
    /// often it invokes `next_batch` (typically aligned with [`SamplingPolicy::period_ms`]).
    fn next_batch(&mut self) -> Result<Option<MeasurementBatch>> {
        if self.points.is_empty() {
            return Err(AcquisitionError::Source(
                "modbus source has no bound points".to_owned(),
            ));
        }

        let (tag, point) = &self.points[self.next_index];
        let tag = tag.clone();
        let point = point.clone();

        let raw = self.read_raw_register(&point)?;
        let value = map_register(raw, &point);
        let timestamp = Timestamp::now();
        let sample = Measurement::new(value, Quality::Good, timestamp);
        let batch = MeasurementBatch::new(tag, vec![sample]);

        self.next_index = (self.next_index + 1) % self.points.len();
        Ok(Some(batch))
    }
}

impl DataSourcePort for ModbusTcpSource {
    fn subscribe(
        &self,
        bindings: &[TagBinding],
        policy: &SamplingPolicy,
    ) -> Result<Box<dyn MeasurementSource>> {
        let points: Vec<(TagId, ModbusPoint)> = bindings
            .iter()
            .map(|binding| {
                let point = parse_modbus_address(binding.address())?;
                Ok((binding.tag().clone(), point))
            })
            .collect::<Result<Vec<_>>>()?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                AcquisitionError::Source(format!("failed to create tokio runtime: {err}"))
            })?;

        let addr = self.addr;
        let client = runtime
            .block_on(async { tcp::connect(addr).await })
            .map_err(|err| AcquisitionError::Source(format!("modbus tcp connect failed: {err}")))?;

        let _period_ms = policy.period_ms();

        Ok(Box::new(ModbusPollSource {
            runtime,
            client,
            points,
            next_index: 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use sp_acquisition::{DataSourcePort as _, TagBinding};
    use sp_kernel::TagId;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio::net::TcpListener;

    use super::*;

    /// Minimal read-only Modbus TCP responder for integration tests.
    ///
    /// The allowed dependency set includes only the `tcp` client feature of
    /// `tokio-modbus`, not `server`, so this hand-rolled responder implements
    /// just enough of FC 0x03 / 0x04 to validate end-to-end reads.
    async fn serve_read_only_modbus(
        listener: TcpListener,
        holding: Arc<HashMap<u16, u16>>,
        input: Arc<HashMap<u16, u16>>,
    ) {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let holding = Arc::clone(&holding);
            let input = Arc::clone(&input);

            tokio::spawn(async move {
                let mut request = [0_u8; 12];
                if socket.read_exact(&mut request).await.is_err() {
                    return;
                }

                let transaction_id = [request[0], request[1]];
                let unit_id = request[6];
                let function = request[7];
                let start = u16::from_be_bytes([request[8], request[9]]);
                let quantity = u16::from_be_bytes([request[10], request[11]]);

                let table = match function {
                    0x03 => &holding,
                    0x04 => &input,
                    _ => return,
                };

                let mut pdu = Vec::with_capacity(2 + usize::from(quantity) * 2);
                pdu.push(function);
                pdu.push((quantity * 2) as u8);
                for offset in 0..quantity {
                    let address = start.saturating_add(offset);
                    let value = table.get(&address).copied().unwrap_or(0);
                    pdu.extend_from_slice(&value.to_be_bytes());
                }

                let length = (pdu.len() + 1) as u16;
                let mut response = Vec::with_capacity(7 + pdu.len());
                response.extend_from_slice(&transaction_id);
                response.extend_from_slice(&0_u16.to_be_bytes());
                response.extend_from_slice(&length.to_be_bytes());
                response.push(unit_id);
                response.extend_from_slice(&pdu);

                socket.write_all(&response).await.ok();
            });
        }
    }

    #[test]
    fn read_holding_register_over_tcp() {
        let server_rt = tokio::runtime::Runtime::new().expect("server runtime");
        let listener = server_rt
            .block_on(TcpListener::bind("127.0.0.1:0"))
            .expect("bind ephemeral port");
        let server_addr = listener.local_addr().expect("local addr");

        let mut holding = HashMap::new();
        holding.insert(0, 1234);
        let holding = Arc::new(holding);
        let input = Arc::new(HashMap::new());

        server_rt.spawn(serve_read_only_modbus(listener, holding, input));

        let tag = TagId::new("PT-1101").expect("tag");
        let binding = TagBinding::new(tag, "holding:0").expect("binding");
        let source = ModbusTcpSource::new(server_addr);
        let mut stream = source
            .subscribe(&[binding], &SamplingPolicy::default())
            .expect("subscribe");

        let batch = stream
            .next_batch()
            .expect("next_batch")
            .expect("live batch");
        assert_eq!(batch.tag().as_str(), "PT-1101");
        assert_eq!(batch.samples().len(), 1);
        assert_eq!(batch.samples()[0].value(), 1234.0);
        assert_eq!(batch.samples()[0].quality(), Quality::Good);
    }

    #[test]
    fn from_str_parses_host_port() {
        let source = "127.0.0.1:1502"
            .parse::<ModbusTcpSource>()
            .expect("from_str");
        assert_eq!(source.addr, "127.0.0.1:1502".parse().expect("addr"));
    }

    #[test]
    fn subscribe_rejects_invalid_binding_address() {
        let tag = TagId::new("PT-1101").expect("tag");
        let binding = TagBinding::new(tag, "foo:bar").expect("binding");
        let source = ModbusTcpSource::new("127.0.0.1:1".parse().expect("addr"));
        let Err(err) = source.subscribe(&[binding], &SamplingPolicy::default()) else {
            panic!("expected invalid address to be rejected");
        };
        assert!(matches!(err, AcquisitionError::Source(_)));
    }
}
