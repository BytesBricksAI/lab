//! `SimPlant` Lab Modbus TCP acquisition adapter (read-only).
//!
//! Implements [`sp_acquisition::DataSourcePort`] by polling holding and input
//! registers over Modbus TCP. This adapter only reads from the field device;
//! it never writes to PLC registers (OT safety).

mod address;
mod tcp_source;

pub use address::{ModbusPoint, RegisterKind, map_register, parse_modbus_address};
pub use tcp_source::ModbusTcpSource;
