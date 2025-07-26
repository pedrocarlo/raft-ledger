use std::collections::HashSet;

use crate::{NodeId, log::Log, state::RaftState};

pub struct Node<T> {
    state: RaftState,
    node_state: NodeState,
    id: NodeId,
    peers: Vec<NodeId>,
    log: Box<dyn Log<T>>,
}

#[derive(Debug, Clone)]
pub enum NodeState {
    Leader,
    Candidate { votes_received: HashSet<NodeId> },
    Follower { current_leader: Option<NodeId> },
}


impl<T> Node<T> {
    fn start_election(&mut self) {
        // self.node_state.
    }
}