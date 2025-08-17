use std::collections::{HashMap, HashSet};

use crate::{Index, NodeId, rpc::VoteRequest, state::RaftState, storage::Storage};

pub struct Node<S: Storage> {
    state: RaftState,
    role: Role,
    id: NodeId,
    peers: Vec<NodeId>,
    storage: S,
    config: Config,
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

impl<S: Storage> Node<S> {
    fn new(id: NodeId, config: Config, storage: S) -> Self {
        Self {
            state: RaftState::default(),
            role: Role::Follower {
                current_leader: None,
            },
            id,
            peers: Vec::new(),
            storage,
            config,
        }
    }

    fn start_election(&mut self) -> Result<(), ()> {
        // TODO: persist state here
        self.state.persistent_state.current_term += 1;
        self.role = Role::Candidate {
            votes_received: HashSet::from_iter(std::iter::once(self.id)),
        };
        self.state.persistent_state.voted_for = Some(self.id);

        let last_term = self.storage.last_term().unwrap_or(0);
        self.state.last_term = last_term;
        let vote_request = VoteRequest {
            term: self.state.persistent_state.current_term,
            candidate_id: self.id,
            log_length: self.storage.last_index().unwrap_or(0),
            last_log_term: last_term,
        };

        let msgs = self.peers.iter().copied();

        Ok(())
    }

    /// Respond to a [Rpc::request_vote] by sending a [VoteResponse]
    fn handle_request_vote(&mut self, request: VoteRequest) {
        let VoteRequest {
            term,
            candidate_id,
            log_length,
            last_log_term,
        } = request;
        if term > self.state.persistent_state.current_term {
            // TODO: persist state here
            self.state.persistent_state.current_term = term;
            self.role = Role::Follower {
                current_leader: Some(candidate_id),
            };
        }
        let last_term = self.storage.last_term().unwrap_or(0);
        let log_length = self.storage.last_index().unwrap_or(0);

        let log_ok =
            (last_log_term > last_term) || (last_log_term == last_term && log_length >= log_length);
        let voted_for = self.state.persistent_state.voted_for;

        if term == self.state.persistent_state.current_term
            && log_ok
            && (voted_for.is_none() || voted_for.is_some_and(|id| id == candidate_id))
        // Voted for the candidate or no one
        {
            // TODO: persist state
            self.state.persistent_state.voted_for = Some(candidate_id);
        } else {
        }
        todo!()
    }
}
