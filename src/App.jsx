import React, { useState, useCallback, useEffect } from "react";
import Sidebar from "./components/Sidebar.jsx";
import ConnectionPanel from "./components/ConnectionPanel.jsx";
import Settings from "./components/Settings.jsx";

// Soft wrapper around the Tauri bridge so the UI also runs in a plain
// browser (e.g. `vite dev`) for fast iteration without the Rust shell.
const invoke = async (cmd, args) => {
  if (window.__TAURI__) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/tauri");
    return tauriInvoke(cmd, args);
  }
  console.debug(`[mock invoke] ${cmd}`, args);
  return mockInvoke(cmd, args);
};

// --- mock data used outside of the Tauri shell -----------------------
const MOCK_SUBSCRIPTIONS = [
  {
    id: "sub-1",
    name: "Основная подписка",
    url: "https://sub.example.com/a1b2c3",
    collapsed: false,
    servers: [
      { id: "srv-1", country: "Германия", flag: "🇩🇪", protocol: "VLESS Reality", ping: 38, lat: 51.1657, lng: 10.4515 },
      { id: "srv-2", country: "Нидерланды", flag: "🇳🇱", protocol: "Hysteria2", ping: 52, lat: 52.1326, lng: 5.2913 },
      { id: "srv-3", country: "Финляндия", flag: "🇫🇮", protocol: "VMess", ping: 140, lat: 61.9241, lng: 25.7482 },
    ],
  },
  {
    id: "sub-2",
    name: "Резервная",
    url: "https://backup.example.net/x9y8",
    collapsed: true,
    servers: [
      { id: "srv-4", country: "Турция", flag: "🇹🇷", protocol: "Trojan", ping: 210, lat: 38.9637, lng: 35.2433 },
      { id: "srv-5", country: "Сингапур", flag: "🇸🇬", protocol: "XHTTP Reality", ping: 310, lat: 1.3521, lng: 103.8198 },
    ],
  },
];

const MOCK_COUNTRY_POOL = [
  ["Франция", "🇫🇷"], ["Испания", "🇪🇸"], ["Польша", "🇵🇱"], ["Швеция", "🇸🇪"],
  ["Канада", "🇨🇦"], ["Япония", "🇯🇵"], ["ОАЭ", "🇦🇪"], ["Италия", "🇮🇹"],
];
const MOCK_PROTOCOL_POOL = ["VLESS Reality", "Hysteria2", "VMess", "Trojan", "XHTTP Reality", "Shadowsocks"];

function mockInvoke(cmd, args) {
  switch (cmd) {
    case "list_subscriptions":
      return Promise.resolve(MOCK_SUBSCRIPTIONS);
    case "connect":
    case "disconnect":
      return Promise.resolve({ ok: true });
    case "add_subscription": {
      // no real Rust backend in plain `vite dev`: fabricate a plausible
      // subscription so the add-subscription flow is still demoable
      const picks = [...MOCK_COUNTRY_POOL].sort(() => Math.random() - 0.5).slice(0, 2 + Math.round(Math.random()));
      const sub = {
        id: "sub-" + Math.random().toString(36).slice(2, 9),
        name: `Подписка (${picks.length} серверов)`,
        url: args?.url ?? "",
        collapsed: false,
        servers: picks.map(([country, flag]) => ({
          id: "srv-" + Math.random().toString(36).slice(2, 9),
          country,
          flag,
          protocol: MOCK_PROTOCOL_POOL[Math.floor(Math.random() * MOCK_PROTOCOL_POOL.length)],
          ping: Math.round(30 + Math.random() * 280),
          lat: (Math.random() - 0.5) * 140,
          lng: Math.random() * 360 - 180,
        })),
      };
      return Promise.resolve(sub);
    }
    case "ping_all":
      return Promise.resolve(null); // caller falls back to local jitter in mock mode
    case "get_stats":
      return Promise.resolve(null); // caller falls back to simulated throughput in mock mode
    default:
      return Promise.resolve(null);
  }
}

export default function App() {
  const [subscriptions, setSubscriptions] = useState(MOCK_SUBSCRIPTIONS);
  const [selectedServerId, setSelectedServerId] = useState(null);
  const [connection, setConnection] = useState({
    status: "disconnected", // disconnected | connecting | connected
    mode: "tun", // tun | proxy
    sessionSeconds: 0,
    down: 0,
    up: 0,
  });
  const [settingsOpen, setSettingsOpen] = useState(false);

  // session timer
  useEffect(() => {
    if (connection.status !== "connected") return;
    const t = setInterval(() => {
      setConnection((c) => ({ ...c, sessionSeconds: c.sessionSeconds + 1 }));
    }, 1000);
    return () => clearInterval(t);
  }, [connection.status]);

  // live throughput while connected: real Clash-API stats from the Rust
  // backend inside Tauri, or a simulated signal in plain-browser dev mode
  useEffect(() => {
    if (connection.status !== "connected") return;
    const t = setInterval(async () => {
      if (window.__TAURI__) {
        const stats = await invoke("get_stats");
        if (stats) {
          setConnection((c) => ({
            ...c,
            down: (stats.down_bytes_per_sec ?? 0) / (1024 * 1024),
            up: (stats.up_bytes_per_sec ?? 0) / (1024 * 1024),
          }));
          return;
        }
      }
      setConnection((c) => ({
        ...c,
        down: Math.max(0, c.down + (Math.random() - 0.5) * 2),
        up: Math.max(0, c.up + (Math.random() - 0.4) * 0.6),
      }));
    }, 1200);
    return () => clearInterval(t);
  }, [connection.status]);

  useEffect(() => {
    invoke("list_subscriptions").then((subs) => {
      if (subs) setSubscriptions(subs);
    });
  }, []);

  const allServers = subscriptions.flatMap((s) => s.servers);
  const selectedServer = allServers.find((s) => s.id === selectedServerId) || null;

  const toggleSubscription = useCallback((subId) => {
    setSubscriptions((subs) =>
      subs.map((s) => (s.id === subId ? { ...s, collapsed: !s.collapsed } : s))
    );
  }, []);

  const removeSubscription = useCallback((subId) => {
    setSubscriptions((subs) => subs.filter((s) => s.id !== subId));
  }, []);

  const selectServer = useCallback((serverId) => {
    setSelectedServerId(serverId);
  }, []);

  const handleConnectToggle = useCallback(async () => {
    if (connection.status === "connected") {
      await invoke("disconnect");
      setConnection((c) => ({ ...c, status: "disconnected", sessionSeconds: 0, down: 0, up: 0 }));
      return;
    }
    if (!selectedServer) return;
    setConnection((c) => ({ ...c, status: "connecting" }));
    await invoke("connect", { serverId: selectedServer.id, mode: connection.mode });
    setConnection((c) => ({ ...c, status: "connected" }));
  }, [connection.status, connection.mode, selectedServer]);

  const setMode = useCallback((mode) => {
    setConnection((c) => ({ ...c, mode }));
  }, []);

  const runAutoPing = useCallback(async () => {
    const updated = await invoke("ping_all");
    if (updated) {
      // real measurement from the Rust backend (TCP-connect timing per server)
      setSubscriptions(updated);
      return;
    }
    // mock/browser mode: jitter existing pings so the UI still demos something
    setSubscriptions((subs) =>
      subs.map((s) => ({
        ...s,
        servers: s.servers.map((srv) => ({
          ...srv,
          ping: Math.max(18, Math.round(srv.ping + (Math.random() - 0.5) * 30)),
        })),
      }))
    );
  }, []);

  const handleAddSubscription = useCallback(async () => {
    const url = window.prompt("Вставьте ссылку на подписку:", "https://");
    if (!url) return;
    try {
      const sub = await invoke("add_subscription", { url });
      if (sub) setSubscriptions((subs) => [...subs, sub]);
    } catch (e) {
      window.alert("Не удалось добавить подписку: " + (e?.message ?? e));
    }
  }, []);

  return (
    <div className="app-shell">
      <Sidebar
        subscriptions={subscriptions}
        selectedServerId={selectedServerId}
        onToggleSubscription={toggleSubscription}
        onRemoveSubscription={removeSubscription}
        onSelectServer={selectServer}
        onAutoPing={runAutoPing}
        onAddSubscription={handleAddSubscription}
      />
      <ConnectionPanel
        server={selectedServer}
        connection={connection}
        onConnectToggle={handleConnectToggle}
        onModeChange={setMode}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      {settingsOpen && <Settings onClose={() => setSettingsOpen(false)} />}
    </div>
  );
}
