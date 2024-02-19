# `loadtimer`

Utility binary to sample the CPU usage of processes with a minimal overhead.

You can use one of the prebuilt binaries from the "Release" section.

Example:

```bash
loadtimer -s 60 -n 3 `pidof logd`
```

Example output:

```bash
loadtimer -s 2 -b 0 -n 3 `pidof logd`
Benchmarking PIDs [229]
3 sample(s) of 60s

| Description | CPU usage % | total mean | utime mean | utime stddev | stime mean | stime stddev |
|        logd |         2.5 |        5.0 |        1.3 |         0.58 |        3.7 |         0.58 |
```

If you want to build from source, e.g. for Android:

```bash
cross build --target aarch64-linux-android --release; adb push target/aarch64-linux-android/release/loadtimer /system/bin/; adb shell chmod a+x /system/bin/loadtimer
```

Usage:

```
> loadtimer --help
Usage: loadtimer [<pids...>] [-s <sample-secs>] [-n <num-samples>]

Measure CPU usage of processes.

Positional Arguments:
  pids              pid of process to monitor

Options:
  -s, --sample-secs sample duration in seconds
  -n, --num-samples number of sample points
  --help            display usage information
```
