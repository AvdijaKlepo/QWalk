/// Engine: owns the quantum walk state, drives the tick loop,
/// and broadcasts snapshots to all connected WebSocket subscribers.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::quantum_walk::{grid_graph, ring_graph, Graph, QuantumWalk};

// ── Public snapshot (sent to SvelteKit on every tick) ────────────────────────

/// Full engine state snapshot serialised to JSON and broadcast over WS.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Tick counter (= quantum walk step number).
    pub step: u64,
    /// Normalised probability for each node, range [0.0, 1.0].
    pub probabilities: Vec<f64>,
    /// Node count (so the frontend can layout the grid).
    pub node_count: usize,
    /// Which topology is active.
    pub topology: TopologyKind,
    /// Tick interval in ms (informational).
    pub tick_ms: u64,
}

// ── Topology config ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TopologyKind {
    Grid { side: usize },
    Ring { nodes: usize, chord_step: usize },
}

impl TopologyKind {
    pub fn build_graph(&self) -> Graph {
        match self {
            TopologyKind::Grid { side } => grid_graph(*side),
            TopologyKind::Ring { nodes, chord_step } => ring_graph(*nodes, *chord_step),
        }
    }

    pub fn start_node(&self) -> usize { 0 }
}

impl Default for TopologyKind {
    fn default() -> Self { TopologyKind::Grid { side: 6 } }
}

// ── Engine config ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub topology: TopologyKind,
    /// How often the walk steps forward.
    pub tick_ms: u64,
    /// Broadcast channel capacity (number of snapshots buffered).
    pub channel_capacity: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            topology: TopologyKind::default(),
            tick_ms: 120,
            channel_capacity: 64,
        }
    }
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Shared engine handle.  Clone freely — it's just Arc pointers.
#[derive(Clone)]
pub struct Engine {
    pub sender: broadcast::Sender<Snapshot>,
    inner: Arc<Mutex<EngineInner>>,
    pub config: Arc<EngineConfig>,
}

struct EngineInner {
    walk: QuantumWalk,
    topology: TopologyKind,
    tick_ms: u64,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        let (sender, _) = broadcast::channel(config.channel_capacity);
        let graph = config.topology.build_graph();
        let start = config.topology.start_node();
        let walk = QuantumWalk::new(graph, start);
        let inner = Arc::new(Mutex::new(EngineInner {
            walk,
            topology: config.topology.clone(),
            tick_ms: config.tick_ms,
        }));
        Self {
            sender,
            inner,
            config: Arc::new(config),
        }
    }

    /// Spawns the background tick loop.  Call once on startup.
    pub fn start(&self) {
        let sender = self.sender.clone();
        let inner = self.inner.clone();

        tokio::spawn(async move {
            info!("Engine tick loop started");
            loop {
                let tick_ms = {
                    let mut guard = inner.lock().await;
                    guard.walk.step();
                    let probs = guard.walk.node_probabilities();
                    let snap = Snapshot {
                        step: guard.walk.step,
                        probabilities: probs,
                        node_count: guard.walk.graph.node_count,
                        topology: guard.topology.clone(),
                        tick_ms: guard.tick_ms,
                    };
                    debug!("step={} node_count={}", snap.step, snap.node_count);
                    // Ignore send errors — no subscribers is fine.
                    let _ = sender.send(snap);
                    guard.tick_ms
                };
                tokio::time::sleep(Duration::from_millis(tick_ms)).await;
            }
        });
    }

    /// Replace topology at runtime (resets the walk).
    pub async fn set_topology(&self, topo: TopologyKind) {
        let mut guard = self.inner.lock().await;
        let graph = topo.build_graph();
        let start = topo.start_node();
        guard.walk = QuantumWalk::new(graph, start);
        guard.topology = topo;
        info!("Topology changed, walk reset");
    }

    /// Change tick speed at runtime.
    pub async fn set_tick_ms(&self, ms: u64) {
        let mut guard = self.inner.lock().await;
        guard.tick_ms = ms;
        info!("Tick interval changed to {}ms", ms);
    }

    /// Subscribe to the broadcast channel.
    pub fn subscribe(&self) -> broadcast::Receiver<Snapshot> {
        self.sender.subscribe()
    }

    /// Latest snapshot without waiting for a tick.
    pub async fn current_snapshot(&self) -> Snapshot {
        let guard = self.inner.lock().await;
        Snapshot {
            step: guard.walk.step,
            probabilities: guard.walk.node_probabilities(),
            node_count: guard.walk.graph.node_count,
            topology: guard.topology.clone(),
            tick_ms: guard.tick_ms,
        }
    }
}
