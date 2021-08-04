use anyhow::Result;
use humantime::Duration;
use std::{fs, str};
use structopt::StructOpt;
use waybar::{History, Loop, Output, Percentage};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "mem", help = "CSS class")]
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
    let classes = vec![opt.class.into()];

    // Set up the history. We'll constrain the sparkline to a maximum of 10
    // historical values.
    let mut history = History::new(opt.sparkline.min(10) as usize);

    Loop::new(
        || {
            let mut available = None;
            let mut total = None;

            for line in str::from_utf8(fs::read("/proc/meminfo")?.as_slice())?.split("\n") {
                if line.starts_with("MemTotal:") {
                    total = Some(line.split_whitespace().nth(1).unwrap().parse::<u64>()?);
                } else if line.starts_with("MemAvailable:") {
                    available = Some(line.split_whitespace().nth(1).unwrap().parse::<u64>()?);
                }

                if available.is_some() && total.is_some() {
                    break;
                }
            }

            let available = available.unwrap() as f64;
            let total = total.unwrap() as f64;
            let perc = Percentage::calculate(total - available, total);

            history.push(perc);
            Ok(Output {
                class: classes.clone(),
                percentage: perc.as_u8().into(),
                tooltip: format!("{}", perc),
                text: history.to_string(|p| p.as_u8()),
            })
        },
        &opt.interval.into(),
    )
    .run()?;

    Ok(())
}
