use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use dbus::{
    blocking::{Proxy, SyncConnection},
    Message,
};
use thiserror::Error;

mod raw {
    include!(concat!(env!("OUT_DIR"), "/swaync.rs"));
}

use raw::OrgErikreiderSwayncCc;

pub struct Client {
    conn: SyncConnection,
    dest: String,
    path: String,
    timeout: Duration,

    done: Arc<AtomicBool>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Self::new_with_args(
            "org.erikreider.swaync.cc",
            "/org/erikreider/swaync/cc",
            Duration::from_secs(5),
        )
    }

    pub fn new_with_args(dest: &str, path: &str, timeout: Duration) -> Result<Self, Error> {
        Ok(Self {
            conn: SyncConnection::new_session()?,
            dest: dest.into(),
            path: path.into(),
            timeout,
            done: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn get_dnd(&self) -> Result<bool, Error> {
        Ok(self.proxy().get_dnd()?)
    }

    pub fn notification_count(&self) -> Result<u32, Error> {
        Ok(self.proxy().notification_count()?)
    }

    pub fn subscribe<F>(&self, f: F) -> Result<(), Error>
    where
        F: Fn(raw::OrgErikreiderSwayncCcSubscribe) -> bool + 'static + Send + Sync,
    {
        // Hard won knowledge somewhat indirectly derived from
        // https://github.com/diwic/dbus-rs/discussions/303 is used in this
        // function: basically, blocking Connection instances don't handle
        // bidirectional communication very well. Implementing non-blocking
        // turned out to be non-trivial, seemingly due to immaturity in the
        // non-blocking signal API, so instead we'll leverage the (relative)
        // cheapness of creating new D-Bus connections and have one per
        // subscription.
        //
        // With that decision made, the only important part is that the
        // match_signal() call must be before the first process() call. The
        // easiest way to ensure that is to avoid spawning the processing thread
        // until after match_signal() is done.

        let conn = Arc::new(SyncConnection::new_session()?);
        let dest = self.dest.clone();
        let path = self.path.clone();
        let timeout = self.timeout;

        let proxy = conn.with_proxy(&dest, &path, timeout);
        proxy.match_signal(
            move |s: raw::OrgErikreiderSwayncCcSubscribe, _: &SyncConnection, _: &Message| f(s),
        )?;

        let conn_proc = conn.clone();
        let done_proc = self.done.clone();
        thread::spawn(move || loop {
            if done_proc.load(Ordering::Relaxed) {
                break;
            }
            conn_proc.process(Duration::from_secs(1)).unwrap();
        });

        Ok(())
    }

    pub fn toggle_dnd(&self) -> Result<(), Error> {
        self.proxy().toggle_dnd()?;
        Ok(())
    }

    pub fn toggle_visibility(&self) -> Result<(), Error> {
        Ok(self.proxy().toggle_visibility()?)
    }

    fn proxy(&self) -> Proxy<&SyncConnection> {
        self.conn.with_proxy(&self.dest, &self.path, self.timeout)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        dbg!("drop");
        self.done.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("D-Bus error")]
    DBus(#[from] dbus::Error),
}
