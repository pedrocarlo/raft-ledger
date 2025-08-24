use std::collections::{HashMap, HashSet, VecDeque};

use bytes::Bytes;

use crate::{
    Index, NodeId,
    log::{Log, LogEntry},
    rpc::{AppendEntriesRequest, Message, MessageType, VoteRequest, VoteResponse},
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
    entries: VecDeque<LogEntry>,
    action: Option<Action>,
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

#[derive(Debug, Clone)]
pub struct Config {
    pub election_timeout: std::time::Duration,
    pub heartbeat_interval: std::time::Duration,
}

pub enum Action {
    ReplicateLog,
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
            entries: VecDeque::new(),
            action: None,
        }
    }

    fn start_election_timer(&mut self) {}

    fn cancel_election_timer(&mut self) {}

    fn init_leader_role(&mut self) {
        // TODO: maybe get this info from persistent state
        let log_length = self.storage.last_index().unwrap_or(0);
        let nodes = self.peers.iter().copied().chain(std::iter::once(self.id));

        self.role = Role::Leader {
            next_index: HashMap::from_iter(nodes.clone().map(|node| (node, log_length))),
            match_index: HashMap::from_iter(nodes.map(|node| (node, 0))),
        }
    }

    async fn replicate_log(&mut self) {
        let Role::Leader {
            match_index: sent_length,
            ..
        } = &mut self.role
        else {
            panic!("replicate_log should be only be called when the node is a leader");
        };
        assert!(self.messages.is_empty());
        let log_len = self.storage.len();
        for follower_id in self.peers.iter() {
            let prefix_len = *sent_length.get(follower_id).unwrap();
            let prefix_term = if prefix_len > 0 {
                // TODO: handle error here
                let entry = self.storage.read_entry(prefix_len).await.unwrap();
                entry.term
            } else {
                0
            };
            let suffix = self
                .storage
                .read_entry_v(prefix_len..log_len)
                .await
                .unwrap();
            let msg = AppendEntriesRequest {
                term: self.state.persistent_state.current_term(),
                leader_id: self.id,
                prev_log_index: prefix_len,
                prev_log_term: prefix_term,
                leader_commit_index: self.state.persistent_state.last_commit_index(),
                entries: suffix,
            };
            self.messages.push(Message::new(
                *follower_id,
                MessageType::AppendEntriesRequest(msg),
            ));
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
        self.start_election_timer();
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
        // TODO: maybe get this info from persistent state
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

    fn handle_response_vote(&mut self, response: VoteResponse, voter_id: NodeId) {
        let VoteResponse { term, vote_granted } = response;
        if term > self.state.persistent_state.current_term() {
            self.state.persistent_state.set_current_term(term);
            self.role = Role::Follower {
                current_leader: None,
            };
            self.state.persistent_state.set_voted_for(None);
            self.cancel_election_timer();
        } else if matches!(self.role, Role::Candidate { .. })
            && term == self.state.persistent_state.current_term()
            && vote_granted
        {
            let Role::Candidate { votes_received } = &mut self.role else {
                unreachable!();
            };
            votes_received.insert(voter_id);
            let majority = ((self.peers.len() + 1) as f64 / 2.0).ceil() as usize;
            if votes_received.len() >= majority {
                self.init_leader_role();
                self.cancel_election_timer();
                // TODO: replicate log
            }
        }
    }

    fn propose(&mut self, message: Vec<u8>) {
        let role = &mut self.role;
        let mut replicate_log = false;
        match role {
            Role::Leader { match_index, .. } => {
                self.entries.push_back(LogEntry {
                    data: Bytes::from_owner(message),
                    term: self.state.persistent_state.current_term(),
                });
                // TODO: self.storage.len() could be mismatched from the actual storage len that should actually be here
                // as the log was not actually appended
                match_index.insert(self.id, self.storage.len() + self.entries.len() as u64);
                replicate_log = true;
            }
            Role::Candidate { .. } => {
                // TODO: should error or panic because we cannot propose when in Candidate
            }
            Role::Follower { current_leader } => {
                // TODO: panic or error if we have no current_leader
                let leader = current_leader.unwrap();
                // TODO forward request to the leader via a FIFO link
            }
        }
        if replicate_log {
            self.action = Some(Action::ReplicateLog);
        }
    }

    fn heartbeat(&mut self) {
        if let Role::Leader { .. } = self.role {
            self.action = Some(Action::ReplicateLog);
        }
    }
}
