/// Discrete-time quantum walk on an undirected graph.
///
/// State space: ℂ^(|E| * 2) — one amplitude per directed edge (u→v and v→u).
/// Coin operator : Grover diffusion coin applied per vertex.
/// Shift operator: moves amplitude along each directed edge.
///
/// After each step the per-node probability is the sum of squared amplitudes
/// of all directed edges that *arrive* at that node.

use std::collections::HashMap;

// ── Complex number (f64 real / imaginary) ────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self { Self { re, im } }
    pub fn norm_sq(self) -> f64 { self.re * self.re + self.im * self.im }

    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
    fn add(self, rhs: Self) -> Self {
        Self { re: self.re + rhs.re, im: self.im + rhs.im }
    }
    fn scale(self, s: f64) -> Self { Self { re: self.re * s, im: self.im * s } }
}

// ── Graph ─────────────────────────────────────────────────────────────────────

/// Undirected edge stored as (min, max) for deduplication.
#[derive(Clone, Debug)]
pub struct Graph {
    pub node_count: usize,
    /// Adjacency list: neighbours[u] = list of v where edge (u,v) exists.
    pub neighbours: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(node_count: usize) -> Self {
        Self {
            node_count,
            neighbours: vec![Vec::new(); node_count],
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.node_count && v < self.node_count);
        if !self.neighbours[u].contains(&v) {
            self.neighbours[u].push(v);
            self.neighbours[v].push(u);
        }
    }

    /// Degree of node u.
    pub fn degree(&self, u: usize) -> usize { self.neighbours[u].len() }
}

// ── Quantum walk state ────────────────────────────────────────────────────────

/// The walk lives in the edge-state Hilbert space.
/// Each directed edge (u → v) has a complex amplitude.
/// Indexed by a flat key: u * N + v  (N = node_count).
pub struct QuantumWalk {
    pub graph: Graph,
    /// amplitude[(u, v)] = ψ(u→v)
    amplitude: HashMap<(usize, usize), Complex>,
    pub step: u64,
}

impl QuantumWalk {
    /// Initialise with a uniform superposition over all directed edges
    /// departing from `start_node`.
    pub fn new(graph: Graph, start_node: usize) -> Self {
        let mut amplitude: HashMap<(usize, usize), Complex> = HashMap::new();
        let deg = graph.degree(start_node);
        if deg > 0 {
            let amp = 1.0 / (deg as f64).sqrt();
            for &v in &graph.neighbours[start_node] {
                amplitude.insert((start_node, v), Complex::new(amp, 0.0));
            }
        }
        Self { graph, amplitude, step: 0 }
    }

    /// One full step: Coin ∘ Shift.
    pub fn step(&mut self) {
        // 1. Coin — Grover diffusion at each vertex.
        //    For each vertex u, collect all incoming amplitudes (w → u),
        //    compute the average, then apply  ψ' = 2*avg - ψ  (Grover coin).
        let mut coined: HashMap<(usize, usize), Complex> =
            HashMap::with_capacity(self.amplitude.len());

        for u in 0..self.graph.node_count {
            let neighbours = self.graph.neighbours[u].clone();
            let d = neighbours.len();
            if d == 0 { continue; }

            // Sum of all amplitudes currently at u (i.e. directed edges  * → u).
            let sum: Complex = neighbours
                .iter()
                .filter_map(|&w| self.amplitude.get(&(w, u)).copied())
                .fold(Complex::default(), |acc, x| acc.add(x));

            let two_over_d = 2.0 / d as f64;

            for &w in &neighbours {
                let psi = self.amplitude.get(&(w, u)).copied().unwrap_or_default();
                // Grover coin: C|ψ⟩ = (2/d)|sum⟩ - |ψ⟩
                let coined_amp = sum.scale(two_over_d).add(psi.scale(-1.0));
                coined.insert((w, u), coined_amp);
            }
        }

        // 2. Shift — move each amplitude along its edge: (u→v) becomes (v→u)?
        //    Standard edge-flip shift: S|u,v⟩ = |v,u⟩ ... wait, that would
        //    just swap. The correct shift for a graph walk:
        //    S maps amplitude on (u→v) to (u→v) after coin, meaning the
        //    walker physically moves: amplitude that was "at u heading to v"
        //    is now "at v heading back or continuing."
        //
        //    We use the standard formulation:
        //      after shift, the amplitude on directed edge (u→v) receives
        //      the coined amplitude that was on (v→u), i.e. the walker
        //      at v heading toward u is now at u heading toward v? No —
        //
        //    Correct standard shift for DTQWs on graphs:
        //      S|u,v⟩ = |v,u⟩  (flip the direction, walker moves from u to v)
        //    So the new amplitude on (v,u) = coined amplitude on (u,v).

        let mut shifted: HashMap<(usize, usize), Complex> =
            HashMap::with_capacity(coined.len());

        for (&(u, v), &amp) in &coined {
            // The walker on edge u→v after the coin now moves to v,
            // and the new edge state is v←u (i.e. key (v,u)).
            shifted.insert((v, u), amp);
        }

        self.amplitude = shifted;
        self.step += 1;
    }

    /// Per-node probability: sum of |ψ(w→u)|² for all w adjacent to u.
    pub fn node_probabilities(&self) -> Vec<f64> {
        let n = self.graph.node_count;
        let mut probs = vec![0.0f64; n];
        for (&(_, v), amp) in &self.amplitude {
            probs[v] += amp.norm_sq();
        }
        // Normalise to [0,1] relative to max (for display purposes).
        let max = probs.iter().cloned().fold(0.0f64, f64::max);
        if max > 0.0 {
            for p in &mut probs { *p /= max; }
        }
        probs
    }

    /// Raw (un-normalised) probabilities — useful for logging / debugging.
    pub fn raw_probabilities(&self) -> Vec<f64> {
        let n = self.graph.node_count;
        let mut probs = vec![0.0f64; n];
        for (&(_, v), amp) in &self.amplitude {
            probs[v] += amp.norm_sq();
        }
        probs
    }
}

// ── Preset topologies ─────────────────────────────────────────────────────────

/// n×n grid graph.
pub fn grid_graph(n: usize) -> Graph {
    let mut g = Graph::new(n * n);
    for row in 0..n {
        for col in 0..n {
            let u = row * n + col;
            if col + 1 < n { g.add_edge(u, row * n + col + 1); } // right
            if row + 1 < n { g.add_edge(u, (row + 1) * n + col); } // down
        }
    }
    g
}

/// Random-ish cycle + chords (small-world feel, deterministic).
pub fn ring_graph(n: usize, chord_step: usize) -> Graph {
    let mut g = Graph::new(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n);
        if chord_step > 1 {
            g.add_edge(i, (i + chord_step) % n);
        }
    }
    g
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probability_sums_to_one_on_line() {
        // 3-node line graph: 0—1—2, start at node 1
        let mut g = Graph::new(3);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        let mut walk = QuantumWalk::new(g, 1);
        let total: f64 = walk.raw_probabilities().iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "initial prob = {total}");
        walk.step();
        let total: f64 = walk.raw_probabilities().iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "after 1 step prob = {total}");
    }

    #[test]
    fn grid_graph_has_correct_node_count() {
        let g = grid_graph(4);
        assert_eq!(g.node_count, 16);
    }
}
