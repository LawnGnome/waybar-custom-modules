use std::process::Command;

use anyhow::Result;
use atoi::atoi;
use humantime::Duration;
use num_format::{SystemLocale, ToFormattedString};
use structopt::StructOpt;
use waybar::{Loop, Output};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "swaync", help = "CSS class")]
    class: String,

    #[structopt(short, long, default_value = "1s", help = "interval between updates")]
    interval: Duration,

    #[structopt(
        short,
        long,
        default_value = "swaync-client",
        help = "path to swaync-client"
    )]
    swaync_client: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let locale = SystemLocale::default()?;

    Loop::new(
        || {
            let output = atoi::<u32>(&Command::new(&opt.swaync_client).arg("-c").output()?.stdout);

            Ok(match output {
                Some(count) if count == 0 => Output {
                    text: "".into(),
                    tooltip: "No notifications".into(),
                    class: format!("{} empty", &opt.class),
                    percentage: 0,
                },
                Some(count) => Output {
                    text: format!("{}", count),
                    tooltip: format!(
                        "{} notification{}",
                        count.to_formatted_string(&locale),
                        if count != 1 { "s" } else { "" }
                    ),
                    class: format!("{} has", &opt.class),
                    percentage: 100,
                },
                None => Output {
                    text: "??".into(),
                    tooltip: "Error calling swaync-client".into(),
                    class: format!("{} error", &opt.class),
                    percentage: 50,
                },
            })
        },
        &opt.interval.into(),
    )
    .run()?;

    Ok(())
}
