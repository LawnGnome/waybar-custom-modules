use anyhow::Result;
use humantime::Duration;
use std::time::Instant;
use std::{fs, str};
use structopt::StructOpt;
use waybar::{History, Loop, Output, Percentage};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "cpu", help = "CSS class")]
    class: String,

    #[structopt(short, long, default_value = "1s", help = "interval between updates")]
    interval: Duration,

    #[structopt(
        short,
        long,
        default_value = "5",
        name = "N",
        help = "enable sparkline with N historical values"
    )]
    sparkline: u8,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Set up the history. We'll constrain the sparkline to a maximum of 10
    // historical values.
    let mut history = History::new(opt.sparkline.min(10) as usize);
    let mut last: Option<Snapshot> = None;

    Loop::new(
        || {
            let current = Snapshot::parse(
                str::from_utf8(fs::read("/proc/stat")?.as_slice())?
                    .split("\n")
                    .filter(|line| line.starts_with("cpu "))
                    .nth(0)
                    .unwrap(),
            )?;
            let mut output = Output {
                text: String::new(),
                tooltip: String::new(),
                class: String::new(),
                percentage: 0,
            };

            if let Some(ref last) = last {
                let perc = Percentage::calculate(
                    (current.used() - last.used()) as f64,
                    (current.total() - last.total()) as f64,
                );
                history.push(perc);

                output.percentage = perc.as_u8().into();
                output.tooltip = format!("{}", perc);
                output.text = history.to_string(|p| p.as_u8());
            }

            last = Some(current);
            Ok(output)
        },
        &opt.interval.into(),
    )
    .run()?;

    Ok(())
}

#[derive(Debug)]
struct Snapshot {
    at: Instant,
    states: Vec<u64>,
}

impl Snapshot {
    fn parse(raw: &str) -> Result<Self> {
        Ok(Self {
            at: Instant::now(),
            states: raw
                .split_whitespace()
                .skip(1)
                .map(|field| Ok(field.parse::<u64>()?))
                .collect::<Result<Vec<u64>>>()?,
        })
    }

    fn idle(&self) -> u64 {
        *self.states.get(3).unwrap()
    }

    fn used(&self) -> u64 {
        self.total() - self.idle()
    }

    fn total(&self) -> u64 {
        self.states.iter().fold(0, |acc, n| acc + *n)
    }
}
