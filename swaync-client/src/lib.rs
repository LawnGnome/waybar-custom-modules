use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use dbus::{
    blocking::{Proxy, SyncConnection},
    Message,
};
use ouroboros::self_referencing;

mod raw {
    include!(concat!(env!("OUT_DIR"), "/swaync.rs"));
}

use raw::OrgErikreiderSwayncCc;

#[self_referencing]
struct ClientImpl {
    conn: SyncConnection,

    #[borrows(conn)]
    #[covariant]
    proxy: Proxy<'this, &'this SyncConnection>,
}

pub struct Client {
    client: Arc<ClientImpl>,
    done: mpsc::Sender<()>,
}

impl Client {
    pub fn new() -> anyhow::Result<Self> {
        Self::new_with_args(
            "org.erikreider.swaync.cc",
            "/org/erikreider/swaync/cc",
            Duration::from_secs(5),
        )
    }

    pub fn new_with_args(dest: &str, path: &str, timeout: Duration) -> anyhow::Result<Self> {
        let conn = SyncConnection::new_session()?;
        let dest = String::from(dest);
        let path = String::from(path);

        let client: Arc<ClientImpl> = Arc::new(
            ClientImplBuilder {
                conn: conn,
                proxy_builder: |conn| conn.with_proxy(dest, path, timeout),
            }
            .build(),
        );

        let (done, rx) = mpsc::channel();
        let conn_proc = client.clone();
        thread::spawn(move || {
            // dbg!("client thread starting");
            loop {
                conn_proc
                    .borrow_conn()
                    .process(Duration::from_secs(1))
                    .unwrap();

                // We should get a message in all normal cases here indicating that
                // the thread should stop, but we'll handle an unexpected
                // disconnection gracefully as well just in case.
                match rx.try_recv() {
                    Ok(_) => {
                        // dbg!("client thread stopping due to done message");
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // dbg!("client thread stopping due to disconnection");
                        break;
                    }
                    _ => {}
                }
                // dbg!("client thread spinning");
            }
            // dbg!("client thread for real exiting");
        });

        Ok(Self { client, done })
    }

    pub fn get_dnd(&self) -> anyhow::Result<bool> {
        Ok(self.client.borrow_proxy().get_dnd()?)
    }

    pub fn notification_count(&self) -> anyhow::Result<u32> {
        Ok(self.client.borrow_proxy().notification_count()?)
    }

    pub fn subscribe<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(raw::OrgErikreiderSwayncCcSubscribe) -> bool + 'static + Send + Sync,
    {
        self.client.borrow_proxy().match_signal(
            move |s: raw::OrgErikreiderSwayncCcSubscribe, _: &SyncConnection, _: &Message| f(s),
        )?;

        Ok(())
    }

    pub fn toggle_dnd(&self) -> anyhow::Result<()> {
        self.client.borrow_proxy().toggle_dnd()?;
        Ok(())
    }

    pub fn toggle_visibility(&self) -> anyhow::Result<()> {
        Ok(self.client.borrow_proxy().toggle_visibility()?)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        dbg!("drop");
        let _ = self.done.send(());
    }
}
