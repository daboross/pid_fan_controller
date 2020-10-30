use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub sample_interval: f32,
    pub heat_pressure_srcs: Vec<HeatPressureSource>,
    pub fans: Vec<Fan>,
}

#[derive(Deserialize)]
pub struct HeatPressureSource {
    pub name: String,
    pub wildcard_path: String,
    #[serde(rename = "PID_params")]
    pub pid_parameters: HeatPressureSourcePidParameters,
}

#[derive(Deserialize)]
pub struct HeatPressureSourcePidParameters {
    pub set_point: f64,
    #[serde(rename = "P")]
    pub p: f64,
    #[serde(rename = "I")]
    pub i: f64,
    #[serde(rename = "D")]
    pub d: f64,
}

#[derive(Deserialize)]
pub struct Fan {
    pub name: String,
    pub wildcard_path: String,
    pub pwm_modes: FanPwmModes,
    pub min_pwm: u32,
    pub max_pwm: u32,
    pub heat_pressure_srcs: Vec<String>,
}

#[derive(Deserialize)]
pub struct FanPwmModes {
    pub manual: u32,
    pub auto: u32,
    pub pwm_mode_wildcard_path: String,
}
