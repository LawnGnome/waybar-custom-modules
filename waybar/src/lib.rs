use anyhow::Result;
use miniserde::{json, Serialize};
use std::thread::sleep;
use std::time::Duration;

mod percentage;
pub use percentage::Percentage;

mod value;
pub use value::History;

#[derive(Debug)]
pub struct Loop<F> {
    callback: F,
    interval: Duration,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: i32,
}

impl<F> Loop<F>
where
    F: FnMut() -> Result<Output>,
{
    pub fn new(callback: F, interval: &Duration) -> Self {
        Self {
            callback,
            interval: interval.clone(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            println!("{}", json::to_string(&(self.callback)()?));
            sleep(self.interval);
        }
    }
}
