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


# PID tuning

* setpoint = 65
* output_limits = 0 - 1
* expect temperature at high load to be 80
  * difference = 85 - 65 = 20
* P, I and D can be configured per heat source


# Control logic

There are two main heat sources:

*   CPU
*   GPU

In order to better utilise the fans and reduce noise, 5 fans need to be controlled differently.

| fan               | control logic                       |
|-------------------|-------------------------------------|
| CPU fan           | CPU PID controller (sp=55)          |
| bottom intake fan | GPU PID controller (sp=65)          |
| middle intake fan | 50% CPU fan + 50% bottom intake fan |
| top intake fan    | CPU fan                             |
| exhaust           | middle intake fan                   |

# Profile the controller

It is important for the system performance that the PID controller is not taking too many CPU cycles.

  python3 -m cProfile pid_fan_controller.py


```
   ncalls  tottime  percall  cumtime  percall filename:lineno(function)
        1    0.001    0.001   56.848   56.848 pid_fan_controller.py:2(<module>)
      285    0.000    0.000    0.008    0.000 pid_fan_controller.py:18(set_speed)
      114    0.000    0.000    0.056    0.000 pid_fan_controller.py:33(read_temp)
       57   56.781    0.996   56.781    0.996 {built-in method time.sleep}
      114    0.000    0.000    0.001    0.000 PID.py:66(__call__)
      228    0.000    0.000    0.000    0.000 PID.py:5(_clamp)
```

We can see the logs from cProfile built-in Python that:

*   The controller spent 99.88% time sleeping (56.781/56.848)
*   There are 57 iterations of always True loop
    *   57 executions of sleep
    *   114 (57*2 temp sensors) executions of read_temp
    *   285 (57*5 fans) execution of set_speed
    *   114 (57*2 PID controllers) PID read calls
    *   228 (114*double sampling rate) PID calculations. By the time this profiling was done, the PID controllers had sample_time set to be 0.5 seconds.
