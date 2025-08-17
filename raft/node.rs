use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use futures_util::{StreamExt, pin_mut, stream::FuturesUnordered};

use crate::{
    Index, NodeId,
    io::IO,
    log::Log,
    rpc::{Rpc, VoteRequest},
    state::RaftState,
};

pub struct Node<I: IO + Rpc<E>, E, L: Log<E>> {
    state: RaftState,
    role: Role,
    id: NodeId,
    peers: Vec<NodeId>,
    log: L,
    _log_type: PhantomData<E>,
    io: I,
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

impl<I: IO + Rpc<E>, E, L: Log<E>> Node<I, E, L> {
    fn new(id: NodeId, config: Config, log: L, io: I) -> Self {
        Self {
            state: RaftState::default(),
            role: Role::Follower {
                current_leader: None,
            },
            id,
            peers: Vec::new(),
            log,
            _log_type: PhantomData,
            config,
            io,
        }
    }

    async fn start_election(&mut self) {
        self.state.persistent_state.current_term += 1;
        self.role = Role::Candidate {
            votes_received: HashSet::from_iter(std::iter::once(self.id)),
        };
        self.state.persistent_state.voted_for = Some(self.id);

        let mut last_term = 0;
        if !self.log.is_empty() {
            last_term = self.log.last_entry_meta().term;
        }
        self.state.last_term = last_term;
        let vote_request = VoteRequest {
            term: self.state.persistent_state.current_term,
            candidate_id: self.id,
            last_log_index: self.log.len() as u64,
            last_log_term: last_term,
        };
        let futs = FuturesUnordered::new();
        self.peers.iter().copied().for_each(|id| {
            let mut io = self.io.clone();
            let request = async move { io.request_vote(vote_request, id).await };
            futs.push(request);
        });
        pin_mut!(futs);
        while let Some(res) = futs.next().await {}
    }
}
