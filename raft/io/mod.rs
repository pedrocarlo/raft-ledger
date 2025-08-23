use std::sync::{Arc, OnceLock};

use bytes::{Bytes, BytesMut};

use crate::rpc::Message;

#[derive(Debug, Clone, Copy)]
pub enum CompletionError {
    IOError,
    Aborted,
}

type CompletionResult = Result<Message, CompletionError>;

#[derive(Debug, Clone)]
pub struct Completion {
    inner: Arc<CompletionInner>,
}

#[derive(Debug)]
struct CompletionInner {
    result: OnceLock<CompletionResult>,
}

impl Completion {
    fn result(&self) -> Option<&CompletionResult> {
        self.inner.result.get()
    }

    fn is_completed(&self) -> bool {
        self.result().is_some_and(|res| res.is_ok())
    }

    fn is_error(&self) -> bool {
        self.result().is_some_and(|res| res.is_err())
    }
}

/// Schdeules I/O
pub trait Scheduler: Clone {
    // TODO: should return the message
    async fn wait_for_completion(&self, c: Completion) -> CompletionResult {
        loop {
            if let Some(res) = c.result() {
                return res.clone();
            } else {
                futures_util::pending!()
            }
        }
    }

    // Network
    async fn recv(&self, buf: BytesMut) -> Completion;
    async fn send(&self, buf: Bytes) -> Completion;

    // File
    async fn read(&self, buf: BytesMut, offset: usize) -> Completion;
    async fn write(&self, buf: Bytes, offset: usize) -> Completion;
}
