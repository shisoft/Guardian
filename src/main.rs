extern crate clap;
extern crate parking_lot;
extern crate stopwatch;
extern crate procinfo;

use std::sync::mpsc::{channel};
use std::sync::Arc;
use std::process::{Command, Stdio};
use std::thread;
use std::cmp::max;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use parking_lot::Mutex;
use clap::{Arg, App, SubCommand};
use stopwatch::{Stopwatch};
use procinfo::pid::{statm};

#[derive(Debug)]
enum TerminationState {
    Timeout,
    Error(io::Error),
    Exited(ExitStatus)
}

#[derive(Debug, Copy, Clone)]
struct Statm {
    size: usize,
    resident: usize,
    share: usize,
    text: usize,
    data: usize
}

#[derive(Debug)]
struct ExitStatus {
    statm: Statm,
    stdout: String,
    stderr: String,
    code: i32
}

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
            .arg(Arg::with_name("ARGUMENTS")
                .help("Sets the arguments to execute")
                .required(false)
                .index(2)
                .multiple(true))
            .get_matches();

    let mut timeout = 0;
    let mut sample_rate = 0;
    let command = matches.value_of("COMMAND").unwrap().to_string();
    let arguments = matches.values_of("ARGUMENTS")
        .map(|vs|
            vs.map(|str|
                str.to_string())
              .collect::<Vec<_>>());
    if let Some(t) = matches.value_of("timeout") {
        timeout = t.parse().unwrap();
    }
    if let Some(s) = matches.value_of("sample_rate") {
        sample_rate = s.parse().unwrap();
    }

    let (term_sender, term_receiver) = channel();
    let wrapped_sender = Arc::new(Mutex::new(term_sender));
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    if timeout > 0 {
        let wrapped_sender = wrapped_sender.clone();
        thread::spawn(move || {
            thread::sleep_ms(timeout);
            let term_sender = wrapped_sender.lock();
            term_sender.send(TerminationState::Timeout)
        });
    }
    thread::spawn(move || {
        let mut cmd = Command::new(command);
        if let Some(arguments) = arguments {
            for arg in arguments {
                cmd.arg(arg);
            }
        }
        let watch = Stopwatch::start_new();
        let child = {
            match cmd
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let term_sender = wrapped_sender.lock();
                    term_sender.send(TerminationState::Error(e));
                    return;
                }
            }
        };
        let pid = child.id();
        let max_stat = Arc::new(AtomicPtr::new(&mut Statm {
            size: 0, resident: 0, share: 0, text: 0, data: 0
        }));
        let max_stat_clone = max_stat.clone();
        if sample_rate > 0 {
            // sample resource consumption
            thread::spawn(move || {
                while running_clone.load(Ordering::Relaxed) {
                    if let Ok(statm) = statm(pid as i32) {
                        let prev_stat = unsafe { *max_stat.load(Ordering::Relaxed) };
                        max_stat.store(
                            &mut Statm {
                                size: max(prev_stat.size, statm.size),
                                resident: max(prev_stat.resident, statm.resident),
                                share: max(prev_stat.share, statm.share),
                                text: max(prev_stat.text, statm.text),
                                data: max(prev_stat.data, statm.data),
                            }, Ordering::Relaxed
                        )
                    } else {
                        break;
                    }
                    thread::sleep_ms(sample_rate);
                }
            });
        }
        let output = child.wait_with_output().unwrap();
        let term_sender = wrapped_sender.lock();
        term_sender.send(TerminationState::Exited(ExitStatus {
            statm: unsafe { *max_stat_clone.load(Ordering::Relaxed) },
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
            code: output.status.code().unwrap()
        }));
    });

    let state = term_receiver.recv().unwrap();
    running.store(false, Ordering::Relaxed);
    println!("{:?}", state);
}
