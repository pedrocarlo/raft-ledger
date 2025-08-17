use crate::{Index, Term};

pub trait Storage {
    /// Return the last term in the log
    fn last_term(&self) -> Option<Term>;
    /// Return the last index in the log.
    fn last_index(&self) -> Option<Index>;
}
