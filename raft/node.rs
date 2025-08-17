use std::{collections::HashSet, marker::PhantomData};

use crate::{NodeId, log::Log, state::RaftState};

pub struct Node<T, L: Log<T>> {
    state: RaftState,
    node_hierarchy: NodeHierarchy,
    id: NodeId,
    peers: Vec<NodeId>,
    log: L,
    _log_type: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub enum NodeHierarchy {
    Leader,
    Candidate { votes_received: HashSet<NodeId> },
    Follower { current_leader: Option<NodeId> },
}

impl<T, L: Log<T>> Node<T, L> {
    fn start_election(&mut self) {
        // self.node_state.
    }
}
