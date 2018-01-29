extern crate clap;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches =
        App::new("Guardian")
            .version("0.1")
            .author("Shisoft <shisoftgenius@gmail.com>")
            .arg(Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT_IN_MS")
                .help("Sets maximum runtime for the process, will be killed when timeout")
                .takes_value(true))
            .arg(Arg::with_name("sample_rate")
                .short("s")
                .long("sample_rate")
                .value_name("SAMPLE_RATE_IN_MS")
                .help("Sets the sampling rate for resource consumption")
                .takes_value(true))
            .arg(Arg::with_name("COMMAND")
                .help("Sets the command to execute")
                .required(true)
                .index(1))
            .get_matches();
    let mut timeout = 0;
    let mut sample_rate = 0;
    let command = matches.value_of("COMMAND").unwrap();
    if let Some(t) = matches.value_of("timeout") {
        timeout = t;
    }
    if let Some(s) = matches.value_of("sample_rate") {
        sample_rate = s;
    }

}
