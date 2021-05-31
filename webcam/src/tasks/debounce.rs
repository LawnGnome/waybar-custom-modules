use anyhow::Result;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time,
};

pub(crate) struct Debouncer {
    input: UnboundedReceiver<()>,
    output: UnboundedSender<()>,
    timeout: std::time::Duration,
}

impl Debouncer {
    pub(crate) fn new(
        input: UnboundedReceiver<()>,
        output: UnboundedSender<()>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            input,
            output,
            timeout,
        }
    }

    pub(crate) async fn process(&mut self) -> Result<()> {
        let mut pending = false;

        loop {
            match time::timeout(self.timeout, self.input.recv()).await {
                Ok(Some(())) => {
                    // We received a message, so we should check once the debounce
                    // is complete.
                    pending = true;
                }
                Ok(None) => {
                    // Nothing left to read on the stream, so let's return.
                    return Ok(());
                }
                Err(_) => {
                    // We hit the debounce timeout, so let's see if we need to
                    // do anything.
                    if pending {
                        self.output.send(())?;
                    }
                    pending = false;
                }
            }
        }
    }
}
