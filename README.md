# `loadtimer`

Utility binary to sample the CPU usage of binaries with a minimal overhead.

Build and push to an Android system with:

```bash
cross build --target aarch64-linux-android --release; adb push target/aarch64-linux-android/release/loadtimer /system/bin/; adb shell chmod a+x /system/bin/loadtimer
```

Example:

```bash
loadtimer -s 60 -b 0 -n 3 `pidof logd`
```

Example output:

```bash
loadtimer -s 2 -b 0 -n 3 `pidof logd`
Benchmarking PIDs [229]
3 sample(s) of 60s with 0s break in between

| Description         | CPU usage % | total mean | utime mean | utime stddev | stime mean | stime stddev |
| 229, 3 samples, 60s |         2.5 |        5.0 |        1.3 |         0.58 |        3.7 |         0.58 |
```

Usage:

```
> loadtimer --help
Usage: loadtimer [<pids...>] [-s <sample-secs>] [-b <break-secs>] [-n <num-samples>]

Measure CPU usage of processes.

Positional Arguments:
  pids              pid of process to monitor

Options:
  -s, --sample-secs sample duration in seconds
  -b, --break-secs  break seconds
  -n, --num-samples number of sample points
  --help            display usage information
```
