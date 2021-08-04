use std::ffi::OsString;

use anyhow::Result;
use humantime::Duration;
use miniserde::json;

use structopt::StructOpt;
use tokio::{runtime::Runtime, signal::unix::SignalKind, sync::mpsc};
use waybar::Output;

mod tasks;
mod udev;
use crate::udev::Checker;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "1s", help = "debounce time")]
    debounce: Duration,

    #[structopt(
        short,
        long,
        default_value = "uvcvideo",
        parse(from_os_str),
        help = "device driver to match"
    )]
    driver: OsString,

    #[structopt(
        short,
        long,
        default_value = "\u{f03d}",
        help = "output when a webcam is present"
    )]
    found: String,

    #[structopt(
        short,
        long,
        default_value = "\u{f4e2}",
        help = "output when a webcam is not present"
    )]
    not_found: String,

    #[structopt(
        short,
        long,
        default_value = "usb",
        parse(from_os_str),
        help = "subsystem to look for webcams on"
    )]
    subsystem: OsString,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // We want a manual runtime because we need to able to create a blocking
    // task, which the high level API doesn't support.
    let rt = Runtime::new()?;

    // OK, so we're going to set up a few tasks. Basically, we are going to:
    //
    // 1. Read from the raw udev monitor. This is extremely noisy.
    // 2. Debounce those inputs into an update channel that only fires after
    //    udev has had a chance to settle.
    // 3. Install a SIGUSR1 handler that also sends events to the update
    //    channel.
    // 4. Listen on the update channel and then interrogate udev to find out if
    //    there's a video device attached.
    //
    // You may ask why we don't just track the state of devices from the
    // updates. Theoretically, this would work, but we'd have to retain the
    // state of the attached devices locally, since the remove events don't
    // include the driver in use. To set that up, we'd have to interrogate udev
    // for the current devices on startup anyway, so we may as well use the same
    // logic on update and keep this as stateless as possible. udev is pretty
    // efficient, and this is reasonably cheap in practice.
    //
    // Note that we're careful not to pass opt directly into any of these blocks
    // or closures until the last one: since we'll need some fields but not
    // others, the tasks::* types have been designed to take clones or copies of
    // those fields as appropriate outside of the spawned task.

    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    let (update_tx, mut update_rx) = mpsc::unbounded_channel();

    // Set up the monitor adapter to get raw udev events and notify the
    // debouncer when they're received. Note that this needs to be on a blocking
    // pool thread because the underlying API is not async.
    let mut monitor = tasks::MonitorAdapter::new(raw_tx, &opt.subsystem);
    rt.spawn_blocking(move || monitor.block());

    // Set up the debouncer.
    let mut debouncer = tasks::Debouncer::new(raw_rx, update_tx.clone(), opt.debounce.into());
    rt.spawn(async move { Ok::<(), anyhow::Error>(debouncer.process().await?) });

    // Set up the signal handler.
    let signal_handler = tasks::SignalHandler::new(update_tx.clone(), SignalKind::user_defined1());
    rt.spawn(async move { Ok::<(), anyhow::Error>(signal_handler.process().await?) });

    // Immediately queue an update.
    update_tx.send(())?;

    // Finally, block the runtime on the last task, which receives update
    // requests on update_rx, checks if any video devices are attached, and
    // prints the appropriate output for waybar to consume.
    Ok(rt.block_on(async move {
        let mut checker = Checker::new(&opt.subsystem, &opt.driver)?;
        let formatter = Formatter {
            found: opt.found,
            not_found: opt.not_found,
        };

        loop {
            while let Some(_) = update_rx.recv().await {
                formatter.output(checker.has_devices()?);
            }
        }

        // This is unreachable, but required to set the return type of the block
        // and therefore allow the ? operator to work, since Rust cannot
        // currently infer the return type of anonymous async blocks.
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    })?)
}

struct Formatter {
    found: String,
    not_found: String,
}

impl Formatter {
    fn output(&self, found: bool) {
        println!(
            "{}",
            json::to_string(&if found {
                Output {
                    tooltip: "Camera connected".into(),
                    class: vec!["found".into()],
                    percentage: 100,
                    text: self.found.clone(),
                }
            } else {
                Output {
                    tooltip: "Camera not connected".into(),
                    class: vec!["not-found".into()],
                    percentage: 0,
                    text: self.not_found.clone(),
                }
            })
        )
    }
}
