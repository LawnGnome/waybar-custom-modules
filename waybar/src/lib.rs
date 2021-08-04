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
            (self.callback)()?.send();
            sleep(self.interval);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: i32,
}

impl Output {
    pub fn send(&self) {
        println!("{}", self.to_json());
    }

    pub fn to_json(&self) -> String {
        json::to_string(self)
    }
}
