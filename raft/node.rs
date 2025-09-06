use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};

use bytes::Bytes;

use crate::{
    Index, NodeId,
    log::{Log, LogEntry},
    message::{
        AppendEntriesRequest, AppendEntriesResponse, Message, MessageType, VoteRequest,
        VoteResponse,
    },
    state::RaftState,
};

pub struct Node<L: Log> {
    state: RaftState,
    role: Role,
    id: NodeId,
    peers: Vec<NodeId>,
    storage: L,
    config: Config,
    send_messages: Vec<Message>,
    entries: VecDeque<LogEntry>,
    action: Option<Action>,
    /// Indexes of Messages  that are to be delivered to the app outside of Raft context
    message_index: Vec<Index>,
}

impl<L: Log> Debug for Node<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("role", &self.role)
            .field("id", &self.id)
            .field("peers", &self.peers)
            .field("config", &self.config)
            .field("send_messages", &self.send_messages)
            .field("entries", &self.entries)
            .field("action", &self.action)
            .field("message_index", &self.message_index)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Role {
    Leader {
        /// for each server, index of the next log entry
        /// to send to that server
        sent_length: HashMap<NodeId, Index>,
        /// for each server, index of highest log entry
        /// known to be replicated on server
        acked_length: HashMap<NodeId, Index>,
    },
    Candidate {
        votes_received: HashSet<NodeId>,
    },
    Follower {
        current_leader: Option<NodeId>,
    },
}

impl Role {
    pub fn is_leader(&self) -> bool {
        matches!(self, Role::Leader { .. })
    }

    pub fn is_candidate(&self) -> bool {
        matches!(self, Role::Candidate { .. })
    }

    pub fn is_follower(&self) -> bool {
        matches!(self, Role::Follower { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub election_timeout: std::time::Duration,
    pub heartbeat_interval: std::time::Duration,
}

#[derive(Debug, Clone, Copy)]
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
            send_messages: Vec::new(),
            entries: VecDeque::new(),
            action: None,
            message_index: Vec::new(),
        }
    }

    fn start_election_timer(&mut self) {}

    fn cancel_election_timer(&mut self) {}

    fn init_leader_role(&mut self) {
        // TODO: maybe get this info from persistent state
        let log_length = self.storage.last_index().unwrap_or(0);
        let nodes = self.peers.iter().copied().chain(std::iter::once(self.id));

        self.role = Role::Leader {
            sent_length: HashMap::from_iter(nodes.clone().map(|node| (node, log_length))),
            acked_length: HashMap::from_iter(nodes.map(|node| (node, 0))),
        }
    }

    async fn replicate_log(&mut self) -> Result<(), ()> {
        let Role::Leader { sent_length, .. } = &mut self.role else {
            panic!("replicate_log should be only be called when the node is a leader");
        };
        assert!(self.send_messages.is_empty());
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
            let suffix = self.storage.read_entry_v(prefix_len..log_len).await?;
            let msg = AppendEntriesRequest {
                term: self.state.persistent_state.current_term(),
                leader_id: self.id,
                prev_log_index: prefix_len,
                prev_log_term: prefix_term,
                leader_commit_index: self.state.persistent_state.last_commit_index(),
                entries: suffix,
            };
            self.send_messages.push(Message::new(
                *follower_id,
                MessageType::AppendEntriesRequest(msg),
            ));
        }
        Ok(())
    }

    fn start_election(&mut self) {
        assert!(self.send_messages.is_empty());

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

        self.send_messages.extend(msgs);
        self.start_election_timer();
    }

    /// Respond to a [Rpc::request_vote] by sending a [VoteResponse]
    fn handle_request_vote(&mut self, request: VoteRequest) {
        assert!(self.send_messages.is_empty());

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
        self.send_messages.push(msg);
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
            Role::Leader {
                acked_length: match_index,
                ..
            } => {
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

    async fn handle_append_entries_request(
        &mut self,
        request: AppendEntriesRequest,
    ) -> Result<(), ()> {
        let AppendEntriesRequest {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit_index,
        } = request;
        if term > self.state.persistent_state.current_term() {
            self.state.persistent_state.set_current_term(term);
            self.state.persistent_state.set_voted_for(None);
            self.cancel_election_timer();
        }

        if term == self.state.persistent_state.current_term() {
            self.role = Role::Follower {
                current_leader: Some(leader_id),
            };
        }
        let entry = if self.storage.len() >= prev_log_index {
            let entry = self.storage.read_entry(prev_log_index - 1).await.unwrap();
            Some(entry)
        } else {
            None
        };
        let log_ok = entry.is_some_and(|entry| prev_log_index == 0 || entry.term == prev_log_term);
        let (ack, success) = if term == self.state.persistent_state.current_term() && log_ok {
            let ack = prev_log_index + entries.len() as u64;
            self.append_entries(prev_log_index, leader_commit_index, entries)
                .await?;
            (ack, true)
        } else {
            (0, false)
        };
        let msg = AppendEntriesResponse {
            term: self.state.persistent_state.current_term(),
            success,
            ack_index: ack,
        };
        self.send_messages.push(Message {
            node_id: leader_id,
            message: MessageType::AppendEntriesResponse(msg),
        });
        Ok(())
    }

    async fn append_entries(
        &mut self,
        prefix_len: Index,
        leader_commit_index: Index,
        mut suffix: Vec<LogEntry>,
    ) -> Result<(), ()> {
        assert!(self.message_index.is_empty());
        if !suffix.is_empty() && self.storage.len() > prefix_len {
            let index = self.storage.len().min(prefix_len + suffix.len() as u64) - 1;
            // TODO: inefficient because we only need the term number and not the data here
            let entry = self.storage.read_entry(index).await?;
            if entry.term != suffix[(index - prefix_len) as usize].term {
                // ATTENTION only write operation here
                self.storage.truncate(prefix_len - 1).await?;
            }
        }
        if prefix_len + suffix.len() as u64 > self.storage.len() {
            self.entries
                .extend(suffix.drain((self.storage.len() - prefix_len) as usize..suffix.len()));
        }
        if leader_commit_index > self.state.persistent_state.last_commit_index() {
            // Deliver logs to the application
            self.message_index
                .extend(self.state.persistent_state.last_commit_index()..leader_commit_index - 1);

            self.state
                .persistent_state
                .set_last_commit_index(leader_commit_index);
        }
        Ok(())
    }

    async fn handle_append_entries_response(
        &mut self,
        response: AppendEntriesResponse,
        follower_id: NodeId,
    ) -> Result<(), ()> {
        let AppendEntriesResponse {
            term,
            ack_index,
            success,
        } = response;
        if let Role::Leader {
            sent_length,
            acked_length,
        } = &mut self.role
            && term == self.state.persistent_state.current_term()
        {
            if success && ack_index >= acked_length[&follower_id] {
                sent_length.insert(follower_id, ack_index);
                acked_length.insert(follower_id, ack_index);
                self.commit_log_entries();
            } else if sent_length[&follower_id] > 0 {
                let prev = sent_length[&follower_id];
                sent_length.insert(follower_id, prev - 1);
                self.replicate_log().await?;
            }
        } else if term > self.state.persistent_state.current_term() {
            self.state.persistent_state.set_current_term(term);
            self.state.persistent_state.set_voted_for(None);
            self.role = Role::Follower {
                current_leader: None,
            };
            self.cancel_election_timer();
        }
        Ok(())
    }

    /// Any log entries that have been acknowledged by a quorum of nodes are ready to be commited by the leader.
    /// When a log entry is committed, its message is delivered to the application
    fn commit_log_entries(&mut self) {
        let Role::Leader { acked_length, .. } = &mut self.role else {
            unreachable!();
        };
        let commit_length = &mut self.state.last_applied_index;
        while *commit_length < self.storage.len() {
            let mut acks = 0;
            for node in self.peers.iter() {
                if acked_length[node] > *commit_length {
                    acks += 1;
                }
            }
            if acks >= ((self.peers.len() + 1).div_ceil(2)) {
                // deliver logs to application
                self.message_index.push(*commit_length);
                *commit_length += 1;
            } else {
                break;
            }
        }
    }
}
