# `loadtimer`

Utility binary to sample the CPU usage of processes with a minimal overhead.

You can use one of the prebuilt binaries from the "Release" section.

Example:

```bash
loadtimer -s 60 -n 3 `pidof logd`
```

Example output:

```bash
loadtimer -s 10 -n 2 `pidof logd`
Measuring CPU usage of PIDs [207]
2 sample(s) of 10s

| Process | CPU % | CPU % stddev | total mean | total stddev | utime mean | stime mean |
|    logd |   1.7 |         0.15 |       17.5 |          1.5 |        1.5 |       16.0 |
```

If you want to build from source, e.g. for Android:

```bash
cross build --target aarch64-linux-android --release; adb push target/aarch64-linux-android/release/loadtimer /system/bin/; adb shell chmod a+x /system/bin/loadtimer
```

Usage:

```
> loadtimer --help
Usage: loadtimer [<pids...>] [-s <sample-secs>] [-n <num-samples>] [-i]

Measure CPU usage of processes.

Positional Arguments:
  pids              PIDs of processes to measure

Options:
  -s, --sample-secs sample duration in seconds
  -n, --num-samples number of sample points
  -i, --interactive run in interactive mode
  --help            display usage information
```
