use std::collections::vec_deque::{Iter, VecDeque};
use std::fmt::Display;

enum HistoryType<T> {
    Single(T),
    Multiple(BoundedDeque<T>),
}

pub struct History<T>(HistoryType<T>);

impl<T> History<T> {
    pub fn new(capacity: usize) -> Self
    where
        T: Default,
    {
        match capacity {
            0 | 1 => Self(HistoryType::Single(T::default())),
            _ => Self(HistoryType::Multiple(BoundedDeque::new(capacity))),
        }
    }

    pub fn push(&mut self, value: T) -> Option<T>
    where
        T: Clone,
    {
        match &mut self.0 {
            HistoryType::Single(v) => {
                let old = v.clone();
                self.0 = HistoryType::Single(value);
                Some(old)
            }
            HistoryType::Multiple(history) => history.push(value),
        }
    }

    pub fn to_string<F>(&self, norm: F) -> String
    where
        T: Ord + Display,
        F: Fn(&T) -> u8,
    {
        match &self.0 {
            HistoryType::Single(v) => format!("{}", v),
            HistoryType::Multiple(history) => format!(
                "{{{}}}",
                history
                    .iter()
                    .map(|v| format!("{}", norm(v).max(0).min(100)))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        }
    }
}

struct BoundedDeque<T> {
    queue: VecDeque<T>,
    capacity: usize,
}

impl<T> BoundedDeque<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, value: T) -> Option<T> {
        if self.queue.len() >= self.capacity {
            let removed = self.queue.pop_front();
            self.queue.push_back(value);
            removed
        } else {
            self.queue.push_back(value);
            None
        }
    }

    fn iter(&self) -> Iter<T> {
        self.queue.iter()
    }
}
