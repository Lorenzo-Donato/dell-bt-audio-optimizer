use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerControlState {
    Auto,
    On,
    Unknown,
}

impl PowerControlState {
    pub fn is_power_saving_active(&self) -> bool {
        matches!(self, PowerControlState::Auto)
    }
}

impl From<&str> for PowerControlState {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "auto" => PowerControlState::Auto,
            "on" => PowerControlState::On,
            _ => PowerControlState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequencyBand {
    TwoPointFourGHz,
    FiveGHz,
    Unknown,
}

impl FrequencyBand {
    pub fn is_coexistence_hazard(&self) -> bool {
        matches!(self, FrequencyBand::TwoPointFourGHz)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FrequencyBand::TwoPointFourGHz => "2.4 GHz",
            FrequencyBand::FiveGHz => "5 GHz",
            FrequencyBand::Unknown => "Desconhecida",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BluetoothCodec {
    Aac,
    Sbc,
    SbcXq,
    HeadsetMono,
    Other,
}

impl BluetoothCodec {
    pub fn is_high_bandwidth_risk(&self) -> bool {
        matches!(self, BluetoothCodec::SbcXq)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BluetoothCodec::Aac => "AAC (Alta fidelidade psicoacústica - Recomendado)",
            BluetoothCodec::Sbc => "SBC (Padrão A2DP estável)",
            BluetoothCodec::SbcXq => "SBC-XQ (Taxa excessiva - Risco de gargalo)",
            BluetoothCodec::HeadsetMono => "HSP/HFP (Mono 8kHz - Modo chamada telefônica)",
            BluetoothCodec::Other => "Outro codec",
        }
    }
}

impl From<&str> for BluetoothCodec {
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("sbc_xq") {
            BluetoothCodec::SbcXq
        } else if lower.contains("aac") {
            BluetoothCodec::Aac
        } else if lower.contains("sbc") {
            BluetoothCodec::Sbc
        } else if lower.contains("headset") || lower.contains("hsp") || lower.contains("hfp") {
            BluetoothCodec::HeadsetMono
        } else {
            BluetoothCodec::Other
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    Good,
    Info,
    Warning,
    Critical,
}

impl IssueSeverity {
    pub fn tag(&self) -> &'static str {
        match self {
            IssueSeverity::Good => "OK",
            IssueSeverity::Info => "INFO",
            IssueSeverity::Warning => "ATENÇÃO",
            IssueSeverity::Critical => "CRÍTICO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_control_state() {
        assert!(PowerControlState::from("auto").is_power_saving_active());
        assert!(!PowerControlState::from("on").is_power_saving_active());
        assert!(!PowerControlState::from("unknown").is_power_saving_active());
    }

    #[test]
    fn test_frequency_band_coexistence() {
        assert!(FrequencyBand::TwoPointFourGHz.is_coexistence_hazard());
        assert!(!FrequencyBand::FiveGHz.is_coexistence_hazard());
    }

    #[test]
    fn test_bluetooth_codec_risk() {
        assert!(BluetoothCodec::from("a2dp-sink-sbc_xq").is_high_bandwidth_risk());
        assert!(!BluetoothCodec::from("a2dp-sink-sbc").is_high_bandwidth_risk());
        assert!(!BluetoothCodec::from("a2dp-sink").is_high_bandwidth_risk());
    }
}
