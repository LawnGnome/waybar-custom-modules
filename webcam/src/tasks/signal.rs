use anyhow::Result;
use tokio::{
    signal::unix::{self, SignalKind},
    sync::mpsc::UnboundedSender,
};

pub(crate) struct SignalHandler {
    output: UnboundedSender<()>,
    signal: SignalKind,
}

impl SignalHandler {
    pub(crate) fn new(output: UnboundedSender<()>, signal: SignalKind) -> Self {
        Self { output, signal }
    }

    pub(crate) async fn process(&self) -> Result<()> {
        let mut signal = unix::signal(self.signal)?;

        loop {
            signal.recv().await;
            self.output.send(())?;
        }
    }
}
