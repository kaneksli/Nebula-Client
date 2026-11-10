import React, { useState } from "react";

function pingColor(ping) {
  if (ping <= 80) return "var(--ping-good)";
  if (ping <= 200) return "var(--ping-mid)";
  return "var(--ping-bad)";
}

function ServerRow({ server, selected, onSelect }) {
  return (
    <button
      className={`server-row ${selected ? "selected" : ""}`}
      onClick={() => onSelect(server.id)}
    >
      <span className="server-flag">{server.flag}</span>
      <span className="server-main">
        <span className="server-name">{server.country}</span>
        <span className="server-protocol">{server.protocol}</span>
      </span>
      <span className="server-ping" style={{ color: pingColor(server.ping) }}>
        {server.ping} мс
      </span>
    </button>
  );
}

function SubscriptionCard({ sub, selectedServerId, onToggle, onRemove, onSelectServer }) {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div
      className="sub-card"
      onContextMenu={(e) => {
        e.preventDefault();
        setMenuOpen(true);
      }}
    >
      <div className="sub-header" onClick={() => onToggle(sub.id)}>
        <span className={`chevron ${sub.collapsed ? "collapsed" : ""}`}>⌄</span>
        <div className="sub-header-text">
          <div className="sub-name">{sub.name}</div>
          <div className="sub-url">{sub.url}</div>
        </div>
        <button
          className="icon-btn"
          title="Обновить подписку"
          onClick={(e) => {
            e.stopPropagation();
            console.log("refresh", sub.id);
          }}
        >
          ↻
        </button>
      </div>

      {!sub.collapsed && (
        <div className="sub-servers">
          {sub.servers.map((server) => (
            <ServerRow
              key={server.id}
              server={server}
              selected={server.id === selectedServerId}
              onSelect={onSelectServer}
            />
          ))}
        </div>
      )}

      {menuOpen && (
        <>
          <div className="menu-backdrop" onClick={() => setMenuOpen(false)} />
          <div className="context-menu">
            <button
              className="context-menu-item danger"
              onClick={() => {
                onRemove(sub.id);
                setMenuOpen(false);
              }}
            >
              Удалить подписку
            </button>
          </div>
        </>
      )}
    </div>
  );
}

export default function Sidebar({
  subscriptions,
  selectedServerId,
  onToggleSubscription,
  onRemoveSubscription,
  onSelectServer,
  onAutoPing,
  onAddSubscription,
}) {
  return (
    <aside className="sidebar">
      <div className="sidebar-top">
        <span className="sidebar-title">Серверы</span>
        <div className="sidebar-top-actions">
          <button className="pill-btn" onClick={onAutoPing}>
            Auto
          </button>
          <button className="icon-btn" title="Добавить подписку" onClick={onAddSubscription}>
            +
          </button>
        </div>
      </div>

      <div className="sub-list">
        {subscriptions.map((sub) => (
          <SubscriptionCard
            key={sub.id}
            sub={sub}
            selectedServerId={selectedServerId}
            onToggle={onToggleSubscription}
            onRemove={onRemoveSubscription}
            onSelectServer={onSelectServer}
          />
        ))}
      </div>

      <style>{`
        .sidebar {
          background: var(--bg-panel);
          border-right: 1px solid var(--line);
          display: flex;
          flex-direction: column;
          min-width: 0;
        }
        .sidebar-top {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 16px 14px 10px;
        }
        .sidebar-title {
          font-size: 13px;
          font-weight: 600;
          color: var(--text-secondary);
          letter-spacing: 0.01em;
        }
        .sidebar-top-actions {
          display: flex;
          gap: 6px;
        }
        .pill-btn {
          background: var(--accent-dim);
          color: var(--accent);
          border: none;
          border-radius: 999px;
          font-size: 12px;
          font-weight: 600;
          padding: 5px 12px;
        }
        .pill-btn:hover { background: rgba(61, 220, 151, 0.26); }
        .icon-btn {
          background: var(--bg-card);
          border: 1px solid var(--line);
          color: var(--text-secondary);
          width: 26px;
          height: 26px;
          border-radius: var(--radius-sm);
          display: inline-flex;
          align-items: center;
          justify-content: center;
          font-size: 14px;
        }
        .icon-btn:hover { background: var(--bg-card-hover); color: var(--text-primary); }

        .sub-list {
          flex: 1;
          overflow-y: auto;
          padding: 4px 10px 14px;
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .sub-card {
          position: relative;
          background: var(--bg-card);
          border: 1px solid var(--line);
          border-radius: var(--radius-md);
          overflow: hidden;
        }
        .sub-header {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px 10px;
          cursor: pointer;
        }
        .chevron {
          color: var(--text-faint);
          transition: transform 0.15s ease;
          font-size: 14px;
          width: 14px;
          text-align: center;
        }
        .chevron.collapsed { transform: rotate(-90deg); }
        .sub-header-text { flex: 1; min-width: 0; }
        .sub-name {
          font-size: 13px;
          font-weight: 600;
          color: var(--text-primary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .sub-url {
          font-size: 11px;
          color: var(--text-faint);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          margin-top: 1px;
        }

        .sub-servers {
          border-top: 1px solid var(--line);
          padding: 4px;
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        .server-row {
          display: flex;
          align-items: center;
          gap: 8px;
          width: 100%;
          background: transparent;
          border: none;
          border-radius: var(--radius-sm);
          padding: 7px 8px;
          text-align: left;
        }
        .server-row:hover { background: var(--bg-card-hover); }
        .server-row.selected {
          background: var(--accent-dim);
        }
        .server-flag { font-size: 16px; line-height: 1; }
        .server-main {
          flex: 1;
          min-width: 0;
          display: flex;
          flex-direction: column;
        }
        .server-name {
          font-size: 12.5px;
          color: var(--text-primary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .server-protocol {
          font-size: 10.5px;
          color: var(--text-faint);
        }
        .server-ping {
          font-family: var(--font-mono);
          font-size: 11px;
          font-weight: 500;
          flex-shrink: 0;
        }

        .menu-backdrop {
          position: fixed;
          inset: 0;
          z-index: 10;
        }
        .context-menu {
          position: absolute;
          top: 36px;
          right: 10px;
          background: var(--bg-card-hover);
          border: 1px solid var(--line-strong);
          border-radius: var(--radius-sm);
          z-index: 11;
          overflow: hidden;
          box-shadow: 0 8px 24px rgba(0,0,0,0.4);
        }
        .context-menu-item {
          display: block;
          width: 100%;
          background: transparent;
          border: none;
          color: var(--text-primary);
          font-size: 12.5px;
          text-align: left;
          padding: 9px 14px;
          white-space: nowrap;
        }
        .context-menu-item:hover { background: rgba(255,255,255,0.06); }
        .context-menu-item.danger { color: var(--ping-bad); }
      `}</style>
    </aside>
  );
}
