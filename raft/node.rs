use std::collections::{HashMap, HashSet};

use crate::{
    Index, NodeId,
    log::Log,
    rpc::{Message, MessageType, VoteRequest, VoteResponse},
    state::RaftState,
};

pub struct Node<L: Log> {
    state: RaftState,
    role: Role,
    id: NodeId,
    peers: Vec<NodeId>,
    storage: L,
    config: Config,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub enum Role {
    Leader {
        /// for each server, index of the next log entry
        /// to send to that server
        next_index: HashMap<NodeId, Index>,
        /// for each server, index of highest log entry
        /// known to be replicated on server
        match_index: HashMap<NodeId, Index>,
    },
    Candidate {
        votes_received: HashSet<NodeId>,
    },
    Follower {
        current_leader: Option<NodeId>,
    },
}

pub struct Config {
    pub election_timeout: std::time::Duration,
    pub heartbeat_interval: std::time::Duration,
}

impl<L: Log> Node<L> {
    fn new(id: NodeId, config: Config, storage: L) -> Self {
        Self {
            state: RaftState::default(),
            role: Role::Follower {
                current_leader: None,
            },
            id,
            peers: Vec::new(),
            storage,
            config,
            messages: Vec::new(),
        }
    }

    fn start_election(&mut self) {
        assert!(self.messages.is_empty());

        // TODO: persist state here
        self.state
            .persistent_state
            .set_current_term(self.state.persistent_state.current_term() + 1);
        self.role = Role::Candidate {
            votes_received: HashSet::from_iter(std::iter::once(self.id)),
        };
        self.state.persistent_state.set_voted_for(Some(self.id));

        let last_term = self.storage.last_term().unwrap_or(0);
        self.state.last_term = last_term;
        let vote_request = VoteRequest {
            term: self.state.persistent_state.current_term(),
            candidate_id: self.id,
            log_length: self.storage.last_index().unwrap_or(0),
            last_log_term: last_term,
        };

        let msgs = self
            .peers
            .iter()
            .copied()
            .map(|id| Message::new(id, MessageType::VoteRequest(vote_request)));

        self.messages.extend(msgs);
        // TODO: start election timer
    }

    /// Respond to a [Rpc::request_vote] by sending a [VoteResponse]
    fn handle_request_vote(&mut self, request: VoteRequest) {
        assert!(self.messages.is_empty());

        let VoteRequest {
            term,
            candidate_id,
            log_length: c_log_length,
            last_log_term,
        } = request;
        if term > self.state.persistent_state.current_term() {
            // TODO: persist state here
            self.state.persistent_state.set_current_term(term);
            self.role = Role::Follower {
                current_leader: Some(candidate_id),
            };
        }
        let last_term = self.storage.last_term().unwrap_or(0);
        let log_length = self.storage.last_index().unwrap_or(0);

        let log_ok = (last_log_term > last_term)
            || (last_log_term == last_term && c_log_length >= log_length);
        let voted_for = self.state.persistent_state.voted_for();

        let vote_granted = if term == self.state.persistent_state.current_term()
            && log_ok
            && (voted_for.is_none() || voted_for.is_some_and(|id| id == candidate_id))
        // Voted for the candidate or no one
        {
            self.state
                .persistent_state
                .set_voted_for(Some(candidate_id));
            true
        } else {
            false
        };
        let msg = Message::new(
            self.id,
            MessageType::VoteResponse(VoteResponse {
                term: self.state.persistent_state.current_term(),
                vote_granted,
            }),
        );
        self.messages.push(msg);
    }
}
