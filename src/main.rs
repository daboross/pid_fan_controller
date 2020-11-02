use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
    time::Instant,
};

use crate::error::*;
use log::{error, info};
use ordered_float::NotNan;
use pid_control::{Controller, PIDController};
use snafu::{ensure, OptionExt, ResultExt};

mod config;
mod error;
mod options;

const DEFAULT_CONFIG_PATH: &str = "/etc/pid_fan_controller_config.yaml";

fn match_single_file_glob(wildcard: &str) -> Result<PathBuf, Error> {
    let mut iter = glob::glob(wildcard).with_context(|| MalformedGlob {
        glob: wildcard.to_owned(),
    })?;
    let val = iter
        .next()
        .with_context(|| GlobMatchesNone {
            glob: wildcard.to_owned(),
        })?
        .with_context(|| GlobError {
            glob: wildcard.to_owned(),
        })?;
    ensure!(
        iter.next().is_none(),
        GlobMatchesMultiple { glob: wildcard }
    );
    Ok(val)
}

fn write_to_fan_file(path: &Path, value: u32) -> Result<(), Error> {
    let ascii = value.to_string();
    std::fs::write(path, &ascii).with_context(|| WritingFile {
        value,
        filename: path.to_owned(),
    })
}

fn read_from_temp_file(path: &Path) -> Result<f64, Error> {
    let value = std::fs::read_to_string(path).with_context(|| ReadingFile {
        filename: path.to_owned(),
    })?;
    let val: u64 = value.trim_end().parse().with_context(|| ParsingResult {
        value,
        filename: path.to_owned(),
    })?;
    Ok(val as f64 / 1000.0)
}

#[derive(Clone)]
struct Fan {
    name: String,
    pwm_path: PathBuf,
    min_pwm: u32,
    max_pwm: u32,
    max_pwm_when_critical: u32,
    heat_sources_indices: Vec<usize>,
    pwm_mode_manual: u32,
    pwm_mode_auto: u32,
    pwm_mode_path: PathBuf,
}

impl Fan {
    fn from_config(
        fan: config::Fan,
        heat_source_name_to_index: &HashMap<&str, usize>,
    ) -> Result<Self, Error> {
        ensure!(
            fan.min_pwm <= fan.max_pwm,
            PwmMinMaxOutOfBounds {
                min_pwm: fan.min_pwm,
                max_pwm: fan.max_pwm,
                name: fan.name,
            }
        );
        Ok(Fan {
            heat_sources_indices: fan
                .heat_pressure_srcs
                .iter()
                .map(|str| {
                    heat_source_name_to_index
                        .get(str.as_str())
                        .copied()
                        .with_context(|| UnknownHeatSource {
                            fan: fan.name.clone(),
                            heat_source: str.clone(),
                        })
                })
                .collect::<Result<_, Error>>()?,
            name: fan.name,
            pwm_path: match_single_file_glob(&fan.wildcard_path)?,
            min_pwm: fan.min_pwm,
            max_pwm: fan.max_pwm,
            max_pwm_when_critical: fan.max_pwm_when_critical.unwrap_or(fan.max_pwm),
            pwm_mode_manual: fan.pwm_modes.manual,
            pwm_mode_auto: fan.pwm_modes.auto,
            pwm_mode_path: match_single_file_glob(&fan.pwm_modes.pwm_mode_wildcard_path)?,
        })
    }

    fn set_control_mode(&self, mode: FanControlMode) -> Result<(), Error> {
        write_to_fan_file(
            &self.pwm_mode_path,
            match mode {
                FanControlMode::Auto => self.pwm_mode_auto,
                FanControlMode::Manual => self.pwm_mode_manual,
            },
        )
    }

    /// Set fan PWM speed. Percentage is allowed to go above 1.0 only when heat
    /// sources have hit emergency temperatures.
    fn set_pwm_speed_percent(&self, percent: f64) -> Result<(), Error> {
        assert!(percent >= 0.0);
        let speed = self
            .max_pwm_when_critical
            .min(self.min_pwm + ((self.max_pwm - self.min_pwm) as f64 * percent).round() as u32);
        write_to_fan_file(&self.pwm_path, speed)?;
        info!("{} = {}", self.name, speed);
        Ok(())
    }

    fn update(&self, heat_sources: &[HeatPressureSource]) -> Result<(), Error> {
        self.set_pwm_speed_percent(
            self.heat_sources_indices
                .iter()
                .copied()
                .map(|idx| heat_sources[idx].output())
                .max()
                .map(Into::into)
                .unwrap_or(1.0),
        )
    }
}

struct HeatPressureSource {
    name: String,
    path: PathBuf,
    pid_controller: PIDController,
    last_update_time: Option<Instant>,
    last_output_value: NotNan<f64>,
    critical_temperature: NotNan<f64>,
}

impl HeatPressureSource {
    fn from_config(source: config::HeatPressureSource) -> Result<Self, Error> {
        let mut pid_controller = PIDController::new(
            source.pid_parameters.p,
            source.pid_parameters.i,
            source.pid_parameters.d,
        );
        pid_controller.set_target(source.pid_parameters.set_point);
        pid_controller.set_limits(0.0, 1.0);
        Ok(HeatPressureSource {
            name: source.name,
            path: match_single_file_glob(&source.wildcard_path)?,
            pid_controller,
            last_update_time: None,
            last_output_value: NotNan::new(1.0).unwrap(),
            critical_temperature: NotNan::new(
                source
                    .pid_parameters
                    .critical_temperature
                    .unwrap_or(std::f64::INFINITY),
            )
            .unwrap(),
        })
    }
    fn update(&mut self) -> Result<(), Error> {
        let now = Instant::now();
        let temp = read_from_temp_file(&self.path)?;
        let delta_t = self
            .last_update_time
            .map(|v| (now - v).as_secs_f64())
            .unwrap_or(0.0);
        self.last_update_time = Some(now);
        if temp > *self.critical_temperature {
            self.pid_controller.set_limits(0.0, std::f64::INFINITY);
        } else {
            // This will result in a sudden fan speed change whenever we leave
            // critical temperature range (thus, if the system is hovering
            // around critical, we'll let it hit the temp, then slowly ramp up
            // fans, then once it falls back down immediately clamp fans back to
            // their regular values).
            //
            // That's kind of the point, though. The critical temperature fan
            // unlimiting is intended as an emergency measure, not for regular use.
            self.pid_controller.set_limits(0.0, 1.0);
        }
        self.last_output_value = NotNan::new(self.pid_controller.update(temp, delta_t))
            .with_context(|| PidControllerNan {
                name: self.name.clone(),
                pid_controller: self.pid_controller.clone(),
            })?;
        info!(
            "{}: read {}, target {} => {}",
            self.name,
            temp,
            self.pid_controller.target(),
            self.last_output_value
        );
        Ok(())
    }

    fn output(&self) -> NotNan<f64> {
        self.last_output_value
    }
}

struct State {
    sample_interval: Duration,
    fans: Vec<Fan>,
    heat_sources: Vec<HeatPressureSource>,
}

#[derive(Copy, Clone)]
enum FanControlMode {
    Auto,
    Manual,
}

impl State {
    pub fn from_config(config: config::Config) -> Result<Self, Error> {
        let sample_interval = Duration::from_secs_f32(config.sample_interval);

        let heat_source_name_to_index = config
            .heat_pressure_srcs
            .iter()
            .enumerate()
            .map(|(idx, src)| (src.name.as_str(), idx))
            .collect::<HashMap<_, _>>();

        let fans = config
            .fans
            .into_iter()
            .map(|fan| Fan::from_config(fan, &heat_source_name_to_index))
            .collect::<Result<_, Error>>()?;

        let heat_sources = config
            .heat_pressure_srcs
            .into_iter()
            .map(HeatPressureSource::from_config)
            .collect::<Result<_, Error>>()?;

        Ok(State {
            sample_interval,
            fans,
            heat_sources: heat_sources,
        })
    }

    pub fn from_file(file: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(file).with_context(|| OpenConfig {
            filename: file.to_owned(),
        })?;
        let config =
            serde_yaml::from_slice::<config::Config>(&bytes).with_context(|| ParseConfig {
                filename: file.to_owned(),
            })?;

        Self::from_config(config)
    }

    pub fn set_fan_control_modes(&self, mode: FanControlMode) -> Result<(), Error> {
        for fan in &self.fans {
            fan.set_control_mode(mode)?;
        }
        Ok(())
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        println!(
            "[{}][{}]: {}",
            record.level(),
            record.target(),
            record.args()
        );
    }
    fn flush(&self) {}
}

fn startup() -> Result<(State, options::Mode), Error> {
    log::set_logger(&Logger).expect("should only startup once");
    log::set_max_level(log::LevelFilter::Error);
    let options = options::get()?;
    if options.verbose {
        log::set_max_level(log::LevelFilter::Info);
    }
    Ok((State::from_file(&options.config_path)?, options.mode))
}

fn main() {
    let (state, mode) = match startup() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error starting program: {}", e);
            std::process::exit(1);
        }
    };

    match mode {
        options::Mode::RunFanControl => run_fan_control(state),
        options::Mode::ResetFanModes => switch_mode_auto(state),
    }
}

fn run_fan_control(mut state: State) {
    match state.set_fan_control_modes(FanControlMode::Manual) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error setting initial control modes: {}", e);
            std::process::exit(2);
        }
    }

    loop {
        for source in &mut state.heat_sources {
            source
                .update()
                .unwrap_or_else(|e| error!("Error updating source {}: {}", source.name, e));
        }
        for fan in &mut state.fans {
            fan.update(&state.heat_sources)
                .unwrap_or_else(|e| error!("Error updating fan {}: {}", fan.name, e));
        }
        std::thread::sleep(state.sample_interval);
    }
}

fn switch_mode_auto(state: State) {
    match state.set_fan_control_modes(FanControlMode::Auto) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error setting control mode back to auto: {}", e);
            std::process::exit(2);
        }
    }
}
