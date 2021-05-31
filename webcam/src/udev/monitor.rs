use std::{ffi::OsStr, io, os::unix::prelude::AsRawFd};

use anyhow::Result;
use nix::poll::{poll, PollFd, PollFlags};
use udev::{MonitorBuilder, MonitorSocket};

/// Monitor wraps the underlying udev monitor API to provide a single blocking
/// call that returns each time an event of interest is returned without
/// interrogating the content of the event. Or, put another way, this adapts
/// udev::MonitorSocket into a simple notification channel, rather than a full
/// reader.
pub(crate) struct Monitor {
    fds: Vec<PollFd>,
    socket: MonitorSocket,
}

impl Monitor {
    pub(crate) fn new<T>(subsystem: T) -> Result<Self>
    where
        T: AsRef<OsStr>,
    {
        let socket = MonitorBuilder::new()?
            .match_subsystem(subsystem)?
            .listen()?;

        let fds = vec![PollFd::new(socket.as_raw_fd(), PollFlags::POLLIN)];

        Ok(Self { fds, socket })
    }

    pub(crate) fn block_until_event(&mut self) -> Result<()> {
        match poll(self.fds.as_mut_slice(), -1)? {
            0 => {
                // This shouldn't happen, since the timeout is set to be
                // infinite, but if it does we can just return and let the
                // caller figure it out.
                Ok(())
            }
            n if n > 0 => {
                // We don't really care about the event, but we have to read it,
                // otherwise poll() will keep telling us about the same event.
                self.socket.next();
                Ok(())
            }
            _ => {
                // We got an error from poll(). This will be reported through
                // errno, which we can grab via the Rust standard library.
                Err(io::Error::last_os_error().into())
            }
        }
    }
}
