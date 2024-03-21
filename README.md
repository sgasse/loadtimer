# `loadtimer`

Utility binary to sample the CPU usage of processes with a minimal overhead.

You can use one of the prebuilt binaries from the "Release" section.

Example:

```bash
loadtimer -s 60 -n 3 -t `pidof logd`
```

Example output:

```bash
loadtimer -s 60 -n 3 -t `pidof logd`
Measuring CPU usage of PIDs [229]
3 sample(s) of 60s

| Process            | CPU % | CPU % stddev | total mean | total stddev | utime mean | stime mean |
| logd               |  1.03 |         0.05 |      10.33 |         0.47 |       1.67 |       8.67 |
|  - logd.daemon     |  0.00 |         0.00 |       0.00 |         0.00 |       0.00 |       0.00 |
|  - logd.reader     |  0.00 |         0.00 |       0.00 |         0.00 |       0.00 |       0.00 |
|  - logd.writer     |  0.33 |         0.05 |       3.33 |         0.47 |       1.33 |       2.00 |
|  - logd.control    |  0.00 |         0.00 |       0.00 |         0.00 |       0.00 |       0.00 |
|  - logd.klogd      |  0.07 |         0.05 |       0.67 |         0.47 |       0.33 |       0.33 |
|  - logd.auditd     |  0.47 |         0.05 |       4.67 |         0.47 |       0.00 |       4.67 |
|  - logd.reader.per |  0.00 |         0.00 |       0.00 |         0.00 |       0.00 |       0.00 |
|  - logd.reader.per |  0.03 |         0.05 |       0.33 |         0.47 |       0.00 |       0.33 |
|  - logd.reader.per |  0.10 |         0.00 |       1.00 |         0.00 |       0.33 |       0.67 |
|  - logd.reader.per |  0.03 |         0.05 |       0.33 |         0.47 |       0.33 |       0.00 |
|  - logd.reader.per |  0.03 |         0.05 |       0.33 |         0.47 |       0.00 |       0.33 |
|  - logd.reader.per |  0.03 |         0.05 |       0.33 |         0.47 |       0.00 |       0.33 |
```

If you want to build from source, e.g. for Android:

```bash
cross build --target aarch64-linux-android --release; adb push target/aarch64-linux-android/release/loadtimer /system/bin/; adb shell chmod a+x /system/bin/loadtimer
```

Usage:

```
> loadtimer --help
Usage: loadtimer [<pids...>] [-s <sample-secs>] [-n <num-samples>] [-i] [-t]

Measure CPU usage of processes.

Positional Arguments:
  pids              PIDs of processes to measure

Options:
  -s, --sample-secs sample duration in seconds
  -n, --num-samples number of sample points
  -i, --interactive run in interactive mode
  -t, --with-threads
                    with threads
  --help            display usage information
```
