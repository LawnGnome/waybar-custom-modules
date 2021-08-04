use anyhow::Result;
use cpu::{Frequency, CPU};
use humantime::Duration;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;
use waybar::{History, Loop, Output, Percentage};

pub mod cpu;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "cpufreq", help = "CSS class")]
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

    #[structopt(
        long,
        default_value = "/sys/devices/system/cpu",
        help = "base path to the CPU sysfs"
    )]
    sysfs_cpu_path: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let classes = vec![opt.class.into()];

    // Enumerate cores.
    let cores = CPU::discover(&opt.sysfs_cpu_path)?;

    // Set up the history. We'll constrain the sparkline to a maximum of 10
    // historical values.
    let mut history = History::new(opt.sparkline.min(10) as usize);

    Loop::new(
        || {
            let freqs = cores
                .iter()
                .map(|core| Ok(core.current_freq()?))
                .collect::<Result<Vec<Frequency>>>()?;
            let core_count = cores.len() as u64;
            let max_freq = max_freq(&cores)?.unwrap_or_else(|| *freqs.iter().max().unwrap());
            let avg_freq = freqs.iter().fold(0, |acc, freq| acc + freq) / core_count;
            let perc = Percentage::calculate(avg_freq as f64, max_freq as f64);

            history.push(perc);

            Ok(Output {
                text: history.to_string(|p| p.as_u8()),
                tooltip: format_tooltip(&freqs),
                class: classes.clone(),
                percentage: perc.as_u8().into(),
            })
        },
        &opt.interval.into(),
    )
    .run()?;

    Ok(())
}

#[derive(Default, Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq)]
struct FrequencyDisplay(Frequency);

impl FrequencyDisplay {
    fn to_string(&self) -> String {
        let f = self.0 as f64;

        if f > 1_000_000. {
            format!("{:.2} GHz", f / 1_000_000.)
        } else if f > 1_000. {
            format!("{:.0} MHz", f / 1_000.)
        } else {
            format!("{} kHz", self.0)
        }
        .to_string()
    }
}

impl std::ops::Deref for FrequencyDisplay {
    type Target = Frequency;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FrequencyDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn format_tooltip(freqs: &Vec<Frequency>) -> String {
    let count = freqs.len();

    format!(
        "{} core{}; ranging from {} to {}",
        count,
        match count {
            1 => "",
            _ => "s",
        },
        FrequencyDisplay(*freqs.iter().min().unwrap()),
        FrequencyDisplay(*freqs.iter().max().unwrap()),
    )
}

fn max_freq(cores: &Vec<CPU>) -> Result<Option<Frequency>> {
    Ok(cores
        .iter()
        .map(|core| Ok(core.max_freq()?))
        .collect::<Result<Vec<Frequency>>>()?
        .into_iter()
        .max())
}
