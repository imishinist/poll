use std::process::Stdio;
use std::{process, thread};

use anyhow::Result;
use clap::Parser;
use humantime::Duration;

#[derive(Debug)]
enum Interval {
    Immediate,
    Delayed(Duration),
}

impl From<Duration> for Interval {
    fn from(duration: Duration) -> Self {
        if duration.as_secs() == 0 {
            Interval::Immediate
        } else {
            Interval::Delayed(duration)
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
struct PollCmd {
    #[clap(long, default_value = "5s")]
    interval: Duration,

    #[clap(short, long = "equals")]
    equals: String,

    #[clap(short, long)]
    on_finish: Option<String>,

    command: String,
}

#[derive(Debug)]
struct Command {
    command: String,
}

impl Command {
    fn new(command: String) -> Self {
        Self { command }
    }

    fn run(&self) -> Result<process::Output> {
        let cmd = process::Command::new("sh")
            .arg("-c")
            .arg(self.command.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(cmd.wait_with_output()?)
    }
}

fn trim_newline(s: &[u8]) -> &[u8] {
    if s.ends_with(&[b'\n']) {
        &s[..s.len() - 1]
    } else {
        s
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let cmd = PollCmd::parse();
    let interval = Interval::from(cmd.interval);

    let command = Command::new(cmd.command);
    log::info!("run: {:?}", command);

    loop {
        let output = command.run()?;
        let status = output.status;

        if status.success() {
            log::info!("status: success");
            let output = trim_newline(&output.stdout);
            if cmd.equals.as_bytes() == output {
                break;
            }
            log::info!(
                "output: {:?} not equals with {:?}",
                output,
                cmd.equals.as_bytes()
            );
        }
        log::info!("exit status: {:?}", status);

        match interval {
            Interval::Immediate => continue,
            Interval::Delayed(duration) => {
                log::info!("waiting interval: {:?}", duration);
                thread::sleep(duration.into());
            }
        }
    }

    if let Some(on_finish) = cmd.on_finish {
        let mut on_finish = process::Command::new("sh")
            .arg("-c")
            .arg(on_finish)
            .spawn()?;
        log::info!("on_finish: {:?}", on_finish);
        let _ = on_finish.wait()?;
    }
    Ok(())
}
