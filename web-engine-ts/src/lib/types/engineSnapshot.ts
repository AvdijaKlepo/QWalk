export interface EngineSnapshot {
    step: number,
    probabilities: number[],
    node_count : number,
    topology : object,
    tick_ms : number
}
