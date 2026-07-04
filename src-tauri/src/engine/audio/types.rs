#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AudioConfig {
    pub driver: String,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub mono: bool,
    pub active_inputs: Option<Vec<i32>>,
    pub active_outputs: Option<Vec<i32>>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            driver: "wasapi".to_string(),
            input_device: None,
            output_device: None,
            sample_rate: 48000,
            buffer_size: 512,
            mono: false,
            active_inputs: None,
            active_outputs: None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AudioDevices {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
    pub input_channels: Vec<String>,
    pub output_channels: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub default: bool,
}
