use std::{thread, time::Duration};

use swaync_client::Client;

fn main() -> anyhow::Result<()> {
    let client = Client::new()?;
    client.subscribe(|s| {
        dbg!(s);
        true
    })?;

    thread::sleep(Duration::from_secs(10));

    Ok(client.toggle_visibility()?)
}
