import type { EngineSnapshot } from "$lib/types/engineSnapshot";

const WS_URL = typeof window !== 'undefined'
    ? `ws://${window.location.hostname}:4000/ws`
    : 'ws://localhost:4000/ws';

const RECONNECT_DELAY_MS = 2_000;



export const engine = $state({
    connected: false,
    reconnectCount: 0,
    snapshot: null as EngineSnapshot | null,
    current: null as EngineSnapshot | null,
});

// $derived still works fine as const exports

let ws : WebSocket;

let reconnectTimer:NodeJS.Timeout | string | number | null | undefined;

function connect() {
    if (typeof window === 'undefined') return; 

    ws = new WebSocket(WS_URL);

    ws.addEventListener('open', () => {
        engine.connected = true;
        engine.reconnectCount = 0;
        console.info('[qwalk] WebSocket connected');
    });

    ws.addEventListener('message', (event) => {
        try {
            const data = JSON.parse(event.data);
            engine.snapshot = data;
        } catch (error) {
            console.warn('[qwalk] Failed to parse message', error);
            scheduleReconnect();
        }
    });

    ws.addEventListener('close', () => {
        engine.connected = false;
        console.info('[qwalk] WebSocket closed - reconnecting...');
    });

    ws.addEventListener('error', () => {
        engine.connected = false;
    });
}

function scheduleReconnect() {
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
    
        engine.reconnectCount++;
        connect();
    }, RECONNECT_DELAY_MS);
}

export function initEngine() {
    connect();
    return () => {
        if (reconnectTimer) clearTimeout(reconnectTimer);
        if (ws) ws.close();
    };
}

const API_BASE = 'http://localhost:4000';


export async function setTopology(kind:string, params:object) {
    const res = await fetch(`${API_BASE}/api/topology`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ kind, ...params})
    });
    return res.json();
}

export async function setSpeed(tick_ms:number) {
    const res = await fetch(`${API_BASE}/api/speed`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tick_ms})
    });
    return res.json();
}