# Guardian
(Not so) lightweight process monitor

## USAGE
First build from source
```
cargo build --release
```

Then goto `target/release` for the binary

With `--help` you can see all the features guardian can offer.

```

Guardian 0.1
Shisoft <shisoftgenius@gmail.com>

USAGE
    guardian [OPTIONS] [--] <COMMAND>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --consumption <CONSUMPTION_OUT>      Sets the consumption output file
    -s, --sample_rate <SAMPLE_RATE_IN_MS>    Sets the sampling rate for resource consumption
    -e, --stderr <STDERR_FILE>               Sets the standard error output file
    -i, --stdin <STDIN_FILE>                 Sets the standard input file
    -o, --stdout <STDOUT_FILE>               Sets the standard output file
    -t, --timeout <TIMEOUT_IN_MS>            Sets maximum runtime for the process, will be killed when timeout

ARGS:
    <COMMAND>...    Sets the command to execute
```
