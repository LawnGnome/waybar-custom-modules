use std::ffi::OsString;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::udev::Monitor;

pub(crate) struct MonitorAdapter {
    sender: UnboundedSender<()>,
    subsystem: OsString,
}

impl MonitorAdapter {
    pub(crate) fn new(sender: UnboundedSender<()>, subsystem: &OsString) -> Self {
        Self {
            sender,
            subsystem: subsystem.clone(),
        }
    }

    pub(crate) fn block(&mut self) -> Result<()> {
        let mut monitor = Monitor::new(&self.subsystem)?;

        loop {
            monitor.block_until_event()?;
            self.sender.send(())?;
        }
    }
}
