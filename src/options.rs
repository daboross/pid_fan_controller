use std::path::PathBuf;

use snafu::ResultExt;

use crate::error::*;

pub struct Options {
    pub config_path: PathBuf,
    pub mode: Mode,
    pub verbose: bool,
}

pub enum Mode {
    ResetFanModes,
    RunFanControl,
}

pub fn get() -> Result<Options, Error> {
    let mut options = getopts::Options::new();
    options.optopt(
        "c",
        "config",
        &format!(
            "set config file. If not set, tries to retrieve PID_FAN_CONFIG_FILE env. \
            If not present, defaults to {}.",
            crate::DEFAULT_CONFIG_PATH
        ),
        "FILE",
    );
    options.optflag("v", "verbose", "enable verbose logging");
    options.optflag(
        "",
        "run-fan-control",
        "switches fans to 'manual' control mode, then runs the fan control service. \
        After this process exits, you must run pid_fan_controller --reset-fan-mode. \
        If not run, fans will be left uncontrolled and this may lead to overheating.",
    );
    options.optflag(
        "",
        "reset-fan-modes",
        "switches fan control to 'auto' (motherboard control) and exits.",
    );
    options.optflag("h", "help", "print this help menu");
    let (program, matches) = {
        let mut args = std::env::args_os();
        (
            args.next().unwrap(),
            options.parse(args).context(GetoptsFailure)?,
        )
    };
    if matches.opt_present("help") {
        eprint!(
            "{}",
            options.usage(&format!("Usage: {} [options]", program.to_string_lossy()))
        );
        std::process::exit(0);
    }
    let mode = match (
        matches.opt_present("run-fan-control"),
        matches.opt_present("reset-fan-modes"),
    ) {
        (true, true) => {
            return Err(Error::MultipleCommands);
        }
        (true, false) => Mode::RunFanControl,
        (false, true) => Mode::ResetFanModes,
        (false, false) => {
            return Err(Error::NoCommands);
        }
    };

    Ok(Options {
        config_path: matches
            .opt_str("config")
            .map(Into::into)
            .or_else(|| std::env::var_os("PID_FAN_CONFIG_FILE").map(Into::into))
            .unwrap_or_else(|| crate::DEFAULT_CONFIG_PATH.into()),
        mode,
        verbose: matches.opt_present("verbose"),
    })
}
