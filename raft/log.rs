use async_trait::async_trait;

use crate::{Index, Term};

pub struct LogEntry<T> {
    data: T,
    term: Term,
    index: Index,
}

#[async_trait]
pub trait Log<T> {
    async fn append(&mut self, data: T);
}
