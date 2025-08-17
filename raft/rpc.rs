use crate::{Index, NodeId, Term, log::LogEntry};

#[derive(Debug, Clone, Copy)]
pub struct VoteRequest {
    /// candidate’s term
    pub term: Term,
    /// candidate requesting vote
    pub candidate_id: NodeId,
    /// index of candidate’s last log entry
    pub last_log_index: Index,
    /// term of candidate’s last log entry
    pub last_log_term: Term,
}

#[derive(Debug, Clone, Copy)]
pub struct VoteResponse {
    /// current_term, for candidate to update itself
    pub term: Term,
    /// true means candidate received vote
    pub vote_granted: bool,
}

pub struct AppendEntriesRequest<'a, T> {
    /// Leader’s term
    pub term: Term,
    /// So follower can redirect clients
    pub leader_id: NodeId,
    /// Index of log entry immediately preceding
    /// new ones
    pub prev_log_index: Index,
    /// Term of `prev_log_index` entry
    pub prev_log_term: Term,
    /// Log entries to store (empty for heartbeat;
    /// may send more than one for efficiency)
    pub entries: &'a [LogEntry<T>],
    /// Leader’s commit_index
    pub leader_commit_index: Index,
}

pub struct AppendEntriesResponse {
    /// current_term, for leader to update itself
    pub term: Term,
    /// true if follower contained entry matching
    /// `prev_log_index` and `prev_log_term`
    pub success: bool,
}

pub trait Rpc<T> {
    /// Invoked by candidates to gather votes
    // TODO: change error type here
    async fn request_vote(
        &mut self,
        request: VoteRequest,
        node: NodeId,
    ) -> Result<VoteResponse, ()>;
    /// Respond to a [Rpc::request_vote] by sending a [VoteResponse]
    // TODO: change error type here
    async fn respond_vote(&mut self, request: VoteRequest, node: NodeId) -> Result<(), ()>;
    /// Invoked by leader to replicate log entries; also used as heartbeat
    async fn request_append_entries<'a>(
        &'a mut self,
        request: AppendEntriesRequest<'a, T>,
        node: NodeId,
    ) -> Result<AppendEntriesResponse, ()>;
    async fn respond_append_entries<'a>(
        &'a mut self,
        request: AppendEntriesRequest<'a, T>,
        node: NodeId,
    ) -> Result<(), ()>;
}
