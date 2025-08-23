use crate::{
    Index, NodeId, Term,
    io::{Completion, Scheduler},
    log::LogEntry,
};

#[derive(Debug, Clone)]
pub struct Message {
    pub node_id: NodeId,
    pub message: MessageType,
}

impl Message {
    pub fn new(node_id: NodeId, message: MessageType) -> Self {
        Self { node_id, message }
    }
}

#[derive(Debug, Clone)]
pub enum MessageType {
    VoteRequest(VoteRequest),
    VoteResponse(VoteResponse),
    AppendEntriesRequest(AppendEntriesRequest),
    AppendEntriesResponse(AppendEntriesResponse),
}

#[derive(Debug, Clone, Copy)]
pub struct VoteRequest {
    /// candidate’s term
    pub term: Term,
    /// candidate requesting vote
    pub candidate_id: NodeId,
    /// index of candidate’s last log entry
    pub log_length: Index,
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

#[derive(Debug, Clone)]
pub struct AppendEntriesRequest {
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
    pub entries: Vec<LogEntry>,
    /// Leader’s commit_index
    pub leader_commit_index: Index,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesResponse {
    /// current_term, for leader to update itself
    pub term: Term,
    /// true if follower contained entry matching
    /// `prev_log_index` and `prev_log_term`
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct Rpc<I: Scheduler> {
    pub io: I,
}

impl<I: Scheduler> Rpc<I> {
    /// Invoked by candidates to gather votes
    // TODO: change error type here
    pub async fn request_vote(&self, request: VoteRequest, node: NodeId) -> Completion {
        todo!()
    }
    /// Respond to a [Rpc::request_vote] by sending a [VoteResponse]
    // TODO: change error type here
    pub async fn respond_vote(&self, respone: VoteResponse, node: NodeId) -> Completion {
        todo!()
    }
    /// Invoked by leader to replicate log entries; also used as heartbeat
    pub async fn request_append_entries(
        &self,
        request: AppendEntriesRequest,
        node: NodeId,
    ) -> Completion {
        todo!()
    }
    pub async fn respond_append_entries<T>(
        &self,
        respone: AppendEntriesResponse,
        node: NodeId,
    ) -> Completion {
        todo!()
    }
}
