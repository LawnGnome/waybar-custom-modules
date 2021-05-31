use std::ffi::OsString;

use anyhow::Result;
use udev::Enumerator;

pub(crate) struct Checker {
    driver: OsString,
    subsystem: OsString,
}

impl Checker {
    pub(crate) fn new(subsystem: &OsString, driver: &OsString) -> Result<Self> {
        Ok(Self {
            subsystem: subsystem.clone(),
            driver: driver.clone(),
        })
    }

    pub(crate) fn has_devices(&mut self) -> Result<bool> {
        let mut enumerator = Enumerator::new()?;
        enumerator.match_subsystem(&self.subsystem)?;
        enumerator.match_property("DRIVER", &self.driver)?;

        Ok(enumerator.scan_devices()?.count() > 0)
    }
}
