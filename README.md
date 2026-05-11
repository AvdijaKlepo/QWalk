```
## Running locally

### 1. Rust engine


cd engine
cargo run 
# Listens on http://localhost:4000
```

First run compiles dependencies (~1 min). Subsequent runs are instant.

**Env vars:**
- `RUST_LOG=qwalk_engine=debug` — verbose logging
- Engine defaults: 6×6 grid, 120ms tick

### 2. SvelteKit frontend


cd web-engine-ts
npm install
npm run dev -- --open
# Opens http://localhost:5173
```

### Snapshot schema (JSON)

```json
{
  "step": 42,
  "probabilities": [0.0, 0.13, 0.87, ...],
  "node_count": 36,
  "topology": { "Grid": { "side": 6 } },
  "tick_ms": 120
}
```

---

## Algorithm

The engine implements a **discrete-time quantum walk (DTQW)** on an undirected
graph, in the edge-state Hilbert space formulation.

- **State space:** one complex amplitude per directed edge (u→v) and (v→u).
- **Coin operator:** Grover diffusion coin applied per vertex — concentrates amplitude back toward the origin (quantum analogue of momentum).
- **Shift operator:** moves amplitude along the edge — S|u,v⟩ = |v,u⟩.
- **Per-node probability:** `p(v) = Σ |ψ(w→v)|²` for all neighbours w.

The walk starts in uniform superposition over edges departing from node 0.

