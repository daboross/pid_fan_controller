use std::path::PathBuf;

use pid_control::PIDController;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Error opening config file from {}: {}", filename.display(), source))]
    OpenConfig {
        filename: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Error parsing YAML from config file {}: {}", filename.display(), source))]
    ParseConfig {
        filename: PathBuf,
        source: serde_yaml::Error,
    },
    #[snafu(display("Fan {} referenced unknown heat source {}", fan, heat_source))]
    UnknownHeatSource { fan: String, heat_source: String },
    #[snafu(display("Couldn't find file matching glob {}", glob))]
    GlobMatchesNone { glob: String },
    #[snafu(display("Multiple conflicting files found for glob {}", glob))]
    GlobMatchesMultiple { glob: String },
    #[snafu(display("Error reading directory {} while searching for files matching glob {}: {}", source.path().display(), glob, source.error()))]
    GlobError {
        glob: String,
        source: glob::GlobError,
    },
    #[snafu(display("Malformed glob pattern {}: {}", glob, source))]
    MalformedGlob {
        glob: String,
        source: glob::PatternError,
    },
    #[snafu(display(
        "Error: pwm_min {} greater than pwm_max {} for fan {}",
        min_pwm,
        max_pwm,
        name
    ))]
    PwmMinMaxOutOfBounds {
        min_pwm: u32,
        max_pwm: u32,
        name: String,
    },
    #[snafu(display(
        "Error writing to fan control file. Tried to write {} to {}, got {}",
        value,
        filename.display(),
        source
    ))]
    WritingFile {
        value: u32,
        filename: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display(
        "Error reading temperature file. Tried to read {}, got {}",
        filename.display(),
        source
    ))]
    ReadingFile {
        filename: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display(
        "Error parsing result from temperature file. Read {} from {}, parsing as integer gave {}",
        value,
        filename.display(),
        source
    ))]
    ParsingResult {
        value: String,
        filename: PathBuf,
        source: std::num::ParseIntError,
    },
    #[snafu(display(
        "PidController for {} gave NaN output. Pid controller state was: {:#?}",
        name,
        pid_controller,
    ))]
    PidControllerNan {
        name: String,
        pid_controller: PIDController,
        source: ordered_float::FloatIsNan,
    },
    #[snafu(display("Error interpreting CLI args: {}", source))]
    GetoptsFailure { source: getopts::Fail },
    #[snafu(display(
        "Error: at most one of --run-fan-control,--switch-mode-auto can be specified"
    ))]
    MultipleCommands,
    #[snafu(display(
        "Error: at least one of --run-fan-control,--switch-mode-auto must be specified"
    ))]
    NoCommands,
}
