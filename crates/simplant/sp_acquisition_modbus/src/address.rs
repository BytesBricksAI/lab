//! Modbus address parsing and deterministic register-to-engineering-value mapping.

use sp_acquisition::{AcquisitionError, Result};

/// Modbus register table kind (read-only access).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    /// Holding register table (function code 0x03).
    Holding,

    /// Input register table (function code 0x04).
    Input,
}

/// A single Modbus point parsed from a tag binding address.
#[derive(Debug, Clone, PartialEq)]
pub struct ModbusPoint {
    kind: RegisterKind,
    register: u16,
    scale: f64,
    offset: f64,
}

impl ModbusPoint {
    /// Register table kind.
    pub fn kind(&self) -> RegisterKind {
        self.kind
    }

    /// Zero-based register index.
    pub fn register(&self) -> u16 {
        self.register
    }

    /// Engineering scale factor applied to the raw u16 value.
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Engineering offset applied after scaling.
    pub fn offset(&self) -> f64 {
        self.offset
    }
}

/// Parses a binding address of the form `"<kind>:<register>[:<scale>[:<offset>]]"`.
pub fn parse_modbus_address(s: &str) -> Result<ModbusPoint> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return Err(invalid_address(
            s,
            "expected kind:register[:scale[:offset]]",
        ));
    }

    let kind = match parts[0] {
        "holding" => RegisterKind::Holding,
        "input" => RegisterKind::Input,
        other => {
            return Err(invalid_address(
                s,
                &format!("unknown register kind '{other}' (expected holding or input)"),
            ));
        }
    };

    let register = parts[1]
        .parse::<u16>()
        .map_err(|_err| invalid_address(s, "register must be a u16"))?;

    let scale = if parts.len() >= 3 {
        parse_finite_f64(parts[2], s, "scale")?
    } else {
        1.0
    };

    let offset = if parts.len() == 4 {
        parse_finite_f64(parts[3], s, "offset")?
    } else {
        0.0
    };

    Ok(ModbusPoint {
        kind,
        register,
        scale,
        offset,
    })
}

/// Maps a raw Modbus register word to an engineering value.
pub fn map_register(raw: u16, point: &ModbusPoint) -> f64 {
    f64::from(raw) * point.scale + point.offset
}

fn parse_finite_f64(raw: &str, address: &str, field: &str) -> Result<f64> {
    let value = raw
        .parse::<f64>()
        .map_err(|_err| invalid_address(address, &format!("{field} must be a finite f64")))?;
    if !value.is_finite() {
        return Err(invalid_address(
            address,
            &format!("{field} must be a finite f64"),
        ));
    }
    Ok(value)
}

fn invalid_address(address: &str, detail: &str) -> AcquisitionError {
    AcquisitionError::Source(format!("invalid modbus address '{address}': {detail}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_address() {
        let point = parse_modbus_address("holding:10:0.01:-40.0").unwrap();
        assert_eq!(point.kind(), RegisterKind::Holding);
        assert_eq!(point.register(), 10);
        assert_eq!(point.scale(), 0.01);
        assert_eq!(point.offset(), -40.0);
    }

    #[test]
    fn parse_defaults_scale_and_offset() {
        let point = parse_modbus_address("input:5").unwrap();
        assert_eq!(point.kind(), RegisterKind::Input);
        assert_eq!(point.register(), 5);
        assert_eq!(point.scale(), 1.0);
        assert_eq!(point.offset(), 0.0);
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_modbus_address("foo:bar").is_err());
        assert!(parse_modbus_address("coil:0").is_err());
        assert!(parse_modbus_address("holding:not_a_number").is_err());
        assert!(parse_modbus_address("holding:0:inf").is_err());
    }

    #[test]
    fn map_register_applies_scale_and_offset() {
        let point = parse_modbus_address("holding:10:0.01:-40.0").unwrap();
        let value = map_register(5000, &point);
        assert!((value - 10.0).abs() < f64::EPSILON);
    }
}
