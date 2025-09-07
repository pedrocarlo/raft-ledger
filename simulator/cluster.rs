use std::{sync::Arc, time::Duration};

use futures_util::stream::FuturesUnordered;
use parking_lot::Mutex;
use raft::{
    NodeId,
    node::{Config, Node},
};
use rand::Rng;
use tokio::task::spawn_local;

use crate::{clock::SimClock, log::SimLog};

pub struct Cluster {
    nodes: Vec<Arc<Mutex<Node<SimLog, SimClock>>>>,
    leader: Option<NodeId>,
}

fn gen_node_config(rng: &mut impl Rng) -> Config {
    Config {
        election_timeout: Duration::from_millis(rng.random_range(100..150)),
        heartbeat_interval: Duration::from_millis(rng.random_range(50..100)),
    }
}

impl Cluster {
    pub fn new(node_count: usize, rng: &mut impl Rng) -> Self {
        assert!(node_count > 2);
        let nodes = (0..node_count)
            .map(|id| {
                Arc::new(Mutex::new(Node::new(
                    id as u64,
                    gen_node_config(rng),
                    SimLog::default(),
                    SimClock,
                )))
            })
            .collect();
        Self {
            nodes,
            leader: None,
        }
    }

    /// Nodes participating in the cluster
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub async fn init_cluster(&mut self) {
        let mut handles = self
            .nodes
            .iter()
            .map(|node| {
                let node = node.clone();
                spawn_local(async move {
                    let mut node = node.lock();
                    node.step().await
                })
            })
            .collect::<Vec<_>>();
    }
}
