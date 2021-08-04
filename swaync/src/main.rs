use std::sync::mpsc;

use anyhow::Result;
use num_format::{SystemLocale, ToFormattedString};
use structopt::StructOpt;
use swaync_client::Client;
use waybar::Output;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "swaync", help = "CSS class")]
    class: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let locale = SystemLocale::default()?;
    let client = Client::new()?;

    output(
        &opt.class,
        &locale,
        client.notification_count()?,
        client.get_dnd()?,
    );

    client.subscribe(move |state| {
        output(&opt.class, &locale, state.count, state.dnd);
        true
    })?;

    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })?;

    Ok(rx.recv()?)
}

fn output(class: &str, locale: &SystemLocale, count: u32, dnd: bool) {
    let classes = format!("{} {}", class, if dnd { "dnd" } else { "disturb" });

    if count == 0 {
        Output {
            text: "".into(),
            tooltip: "No notifications".into(),
            class: format!("{} empty", classes),
            percentage: 0,
        }
    } else {
        Output {
            text: format!("{}", count),
            tooltip: format!(
                "{} notification{}",
                count.to_formatted_string(locale),
                if count != 1 { "s" } else { "" }
            ),
            class: format!("{} has", classes),
            percentage: 100,
        }
    }
    .send();
}
