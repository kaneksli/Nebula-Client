import React from "react";
import Globe from "./Globe.jsx";

function formatDuration(totalSeconds) {
  const h = String(Math.floor(totalSeconds / 3600)).padStart(2, "0");
  const m = String(Math.floor((totalSeconds % 3600) / 60)).padStart(2, "0");
  const s = String(totalSeconds % 60).padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function formatRate(mbps) {
  return `${mbps.toFixed(1)} МБ/с`;
}

export default function ConnectionPanel({ server, connection, onConnectToggle, onModeChange, onOpenSettings }) {
  const { status, mode, sessionSeconds, down, up } = connection;
  const isConnected = status === "connected";
  const isConnecting = status === "connecting";

  return (
    <main className="connection-panel">
      <Globe connectedServer={server} status={status} />

      <button className="settings-fab" onClick={onOpenSettings} title="Настройки">
        ⚙
      </button>

      {isConnected && server && (
        <div className="connected-badge">
          Подключено: {server.country} {server.flag}
        </div>
      )}

      <div className="mode-switch">
        <button
          className={mode === "tun" ? "active" : ""}
          onClick={() => onModeChange("tun")}
        >
          TUN
        </button>
        <button
          className={mode === "proxy" ? "active" : ""}
          onClick={() => onModeChange("proxy")}
        >
          Proxy
        </button>
      </div>

      <div className="connect-cluster">
        <button
          className={`connect-btn ${status}`}
          onClick={onConnectToggle}
          disabled={!server && !isConnected}
        >
          <span className="connect-ring" />
          <span className="connect-label">
            {isConnecting ? "..." : isConnected ? "Отключить" : "Подключить"}
          </span>
        </button>

        <div className="connect-meta">
          <div className="status-line">
            {isConnected ? "Подключено" : isConnecting ? "Подключение…" : "Отключено"}
          </div>
          <div className="timer">{formatDuration(sessionSeconds)}</div>
        </div>
      </div>

      <div className="stats-bar">
        <div className="stat">
          <span className="stat-label">↓</span>
          <span className="stat-value">{formatRate(down)}</span>
        </div>
        <div className="stat">
          <span className="stat-label">↑</span>
          <span className="stat-value">{formatRate(up)}</span>
        </div>
        {server && (
          <div className="stat">
            <span className="stat-label">Пинг</span>
            <span className="stat-value">{server.ping} мс</span>
          </div>
        )}
        {server && (
          <div className="stat">
            <span className="stat-label">Сервер</span>
            <span className="stat-value">{server.country}</span>
          </div>
        )}
      </div>

      <style>{`
        .connection-panel {
          position: relative;
          background: radial-gradient(ellipse at center, #0a0d16 0%, #05060a 70%);
          overflow: hidden;
        }
        .globe-mount {
          position: absolute;
          inset: 0;
        }
        .settings-fab {
          position: absolute;
          top: 18px;
          right: 18px;
          width: 34px;
          height: 34px;
          border-radius: 50%;
          background: rgba(255,255,255,0.06);
          border: 1px solid var(--line);
          color: var(--text-secondary);
          font-size: 15px;
          backdrop-filter: blur(6px);
        }
        .settings-fab:hover { color: var(--text-primary); background: rgba(255,255,255,0.1); }

        .connected-badge {
          position: absolute;
          top: 22px;
          left: 50%;
          transform: translateX(-50%);
          background: rgba(10, 14, 22, 0.75);
          border: 1px solid var(--line-strong);
          backdrop-filter: blur(8px);
          padding: 7px 16px;
          border-radius: 999px;
          font-size: 12.5px;
          font-weight: 500;
          animation: badge-in 0.35s ease;
        }
        @keyframes badge-in {
          from { opacity: 0; transform: translateX(-50%) scale(0.92); }
          to { opacity: 1; transform: translateX(-50%) scale(1); }
        }

        .mode-switch {
          position: absolute;
          top: 18px;
          left: 18px;
          display: flex;
          background: rgba(255,255,255,0.05);
          border: 1px solid var(--line);
          border-radius: 999px;
          padding: 3px;
          backdrop-filter: blur(6px);
        }
        .mode-switch button {
          border: none;
          background: transparent;
          color: var(--text-secondary);
          font-size: 11.5px;
          font-weight: 600;
          letter-spacing: 0.02em;
          padding: 6px 14px;
          border-radius: 999px;
        }
        .mode-switch button.active {
          background: var(--accent);
          color: #04140d;
        }

        .connect-cluster {
          position: absolute;
          left: 50%;
          bottom: 108px;
          transform: translateX(-50%);
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 14px;
        }
        .connect-btn {
          position: relative;
          width: 128px;
          height: 128px;
          border-radius: 50%;
          border: 1px solid var(--line-strong);
          background: rgba(13, 16, 26, 0.55);
          backdrop-filter: blur(4px);
          display: flex;
          align-items: center;
          justify-content: center;
        }
        .connect-btn .connect-label {
          font-size: 13px;
          font-weight: 600;
          color: var(--text-primary);
          z-index: 1;
        }
        .connect-btn.connected {
          border-color: var(--accent);
          box-shadow: 0 0 0 1px var(--accent-dim), 0 0 32px rgba(61, 220, 151, 0.25);
        }
        .connect-btn.connecting .connect-label { color: var(--text-secondary); }
        .connect-ring {
          position: absolute;
          inset: 10px;
          border-radius: 50%;
          border: 1px dashed var(--line-strong);
        }
        .connect-btn.connected .connect-ring {
          border-color: var(--accent-dim);
          animation: pulse-ring 2.4s ease-in-out infinite;
        }
        @keyframes pulse-ring {
          0%, 100% { transform: scale(1); opacity: 0.6; }
          50% { transform: scale(1.06); opacity: 1; }
        }

        .connect-meta { text-align: center; }
        .status-line {
          font-size: 12px;
          color: var(--text-secondary);
        }
        .timer {
          font-family: var(--font-mono);
          font-size: 13px;
          color: var(--text-primary);
          margin-top: 2px;
        }

        .stats-bar {
          position: absolute;
          left: 50%;
          bottom: 28px;
          transform: translateX(-50%);
          display: flex;
          gap: 22px;
          background: rgba(10, 13, 20, 0.6);
          border: 1px solid var(--line);
          backdrop-filter: blur(6px);
          padding: 9px 20px;
          border-radius: var(--radius-lg);
        }
        .stat {
          display: flex;
          flex-direction: column;
          align-items: center;
          min-width: 52px;
        }
        .stat-label {
          font-size: 10px;
          color: var(--text-faint);
        }
        .stat-value {
          font-family: var(--font-mono);
          font-size: 12.5px;
          color: var(--text-primary);
          margin-top: 1px;
          white-space: nowrap;
        }
      `}</style>
    </main>
  );
}
