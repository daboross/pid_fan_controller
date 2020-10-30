# `pid-fan-controller-rs`

This is a fork of [pid_fan_controller](https://github.com/ThunderMikey/pid_fan_controller), with the intention of rewriting in Rust - mainly for my own purposes, and so that I don't need to install any Python libraries globally to effectively use the software.

If you're looking for a better-maintained package with actual updates, I highly suggest you look at the link above. It's the software this was based off of, exactly the same (or more) functionality written in Python. The author has additionally done benchmarking to ensure it uses very little CPU (even being Python, since it's doing practically nothing). The profiling is something I _haven't_ done for this software, so... user beware.

The majority of the rest of this README is copied verbatim from `pid_fan_controller` as of the time of this fork.

# Usage

This project needs Rust and Cargo installed to build. After installing cargo,
use `cargo build --release` to build a release binary. Then, to run the program
manually, use:

```
./target/release/pid_fan_controller --help
```

The intended usage, however, is to run this as a `systemd` service. To do so,
please:
- copy `./target/release/pid_fan_controller` to
  `/usr/local/bin/pid_fan_controller`
- copy `pid-fan-controller.service` and `pid-fan-controller-sleep-hook.service`
  to `/usr/lib/systemd/system/`
- copy `pid_fan_controller_config.yaml` to
  `/etc/pid_fan_controller_config.yaml`.
- enable with

  ```
  sudo systemctl enable pid-fan-controller.service
  sudo systemctl enable pid-fan-controller-sleep-hook.service
  ```

The original project came with a `PKGBUILD` install method. However, I'm
unfamiliar with this format, and do not run an arch linux system (or any system
with `pacman` installed). I've removed it for now, however, if anyone has more
knowledge here and is interested in filing a PR, I'd welcome it back. It just
needs to depend on rust at compile time, and install files appropriately.

# Motivation

The current fan control software (fancontrol from lmsensors) and BIOS built-in “smart fan control” both use linear fan control logics. The fan speed is directly proportional to temperature source.

This leads to abrupt fan speed change on bursty workloads. Usually, the heat generated can be dissipated over time without ramping up fan speed. A linear fan speed controller has no idea of the speed at which temperature changes. A PID controller solves the problem by having a Proportional, Integral and Differential of the temperature history.


# Similar works

- [https://github.com/jbg/macbookfan](https://github.com/jbg/macbookfan)
- [pid_fan_controller](https://github.com/ThunderMikey/pid_fan_controller)
