import React, { useState } from "react";

const SECTIONS = [
  { id: "appearance", label: "Внешний вид" },
  { id: "connection", label: "Подключение" },
  { id: "advanced", label: "Дополнительно" },
];

export default function Settings({ onClose }) {
  const [section, setSection] = useState("appearance");

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="settings-nav">
          {SECTIONS.map((s) => (
            <button
              key={s.id}
              className={section === s.id ? "active" : ""}
              onClick={() => setSection(s.id)}
            >
              {s.label}
            </button>
          ))}
        </div>

        <div className="settings-body">
          {section === "appearance" && (
            <>
              <h3>Фон панели подключения</h3>
              <div className="row">
                <span>Визуализация</span>
                <select defaultValue="globe">
                  <option value="globe">3D-глобус</option>
                  <option value="stars">Звёзды</option>
                  <option value="waves">Волны</option>
                  <option value="gradient">Градиент</option>
                  <option value="particles">Частицы</option>
                  <option value="minimal">Минимализм</option>
                  <option value="custom">Своё изображение</option>
                </select>
              </div>
              <div className="row">
                <span>Цвет акцентного маркера</span>
                <input type="color" defaultValue="#3ddc97" />
              </div>
              <div className="row">
                <span>Скорость вращения (ожидание)</span>
                <input type="range" min="0" max="10" defaultValue="3" />
              </div>
              <div className="row">
                <span>Плотность точек</span>
                <input type="range" min="0" max="10" defaultValue="6" />
              </div>
              <div className="row">
                <span>Тема интерфейса</span>
                <select defaultValue="dark">
                  <option value="dark">Тёмная</option>
                  <option value="light">Светлая</option>
                </select>
              </div>
            </>
          )}

          {section === "connection" && (
            <>
              <h3>Поведение подключения</h3>
              <label className="row checkbox">
                <input type="checkbox" /> Автозапуск с системой
              </label>
              <label className="row checkbox">
                <input type="checkbox" /> Автоподключение при запуске
              </label>
              <label className="row checkbox">
                <input type="checkbox" /> Автообновление подписок по расписанию
              </label>
              <div className="row">
                <span>Ядро</span>
                <select defaultValue="sing-box">
                  <option value="sing-box">sing-box</option>
                  <option value="xray" disabled>
                    Xray-core (скоро)
                  </option>
                </select>
              </div>
              <div className="row">
                <span>Глобальный шорткат</span>
                <input type="text" placeholder="Ctrl+Alt+V" readOnly />
              </div>
            </>
          )}

          {section === "advanced" && (
            <>
              <h3>Диагностика и исключения</h3>
              <div className="row">
                <span>Split-tunneling</span>
                <button className="secondary-btn">Настроить приложения…</button>
              </div>
              <div className="row">
                <span>Логи</span>
                <button className="secondary-btn">Экспортировать логи</button>
              </div>
              <div className="row">
                <span>Обновления приложения</span>
                <button className="secondary-btn">Проверить обновления</button>
              </div>
            </>
          )}
        </div>

        <div className="settings-footer">
          <button className="primary-btn" onClick={onClose}>
            Готово
          </button>
        </div>
      </div>

      <style>{`
        .settings-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0,0,0,0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 50;
        }
        .settings-dialog {
          width: 560px;
          max-width: 92vw;
          max-height: 80vh;
          background: var(--bg-panel);
          border: 1px solid var(--line-strong);
          border-radius: var(--radius-lg);
          display: grid;
          grid-template-columns: 150px 1fr;
          grid-template-rows: 1fr auto;
          overflow: hidden;
        }
        .settings-nav {
          grid-row: 1 / 3;
          background: var(--bg-card);
          border-right: 1px solid var(--line);
          padding: 14px 8px;
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        .settings-nav button {
          background: transparent;
          border: none;
          color: var(--text-secondary);
          text-align: left;
          font-size: 12.5px;
          padding: 8px 10px;
          border-radius: var(--radius-sm);
        }
        .settings-nav button.active {
          background: var(--accent-dim);
          color: var(--accent);
        }
        .settings-body {
          padding: 20px 22px;
          overflow-y: auto;
        }
        .settings-body h3 {
          margin: 0 0 14px;
          font-size: 13px;
          color: var(--text-secondary);
          font-weight: 600;
        }
        .row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 9px 0;
          font-size: 13px;
          border-bottom: 1px solid var(--line);
        }
        .row.checkbox { justify-content: flex-start; gap: 10px; }
        .row select, .row input[type="text"] {
          background: var(--bg-card);
          border: 1px solid var(--line);
          color: var(--text-primary);
          border-radius: var(--radius-sm);
          padding: 5px 8px;
          font-size: 12.5px;
        }
        .secondary-btn {
          background: var(--bg-card);
          border: 1px solid var(--line);
          color: var(--text-primary);
          border-radius: var(--radius-sm);
          padding: 6px 12px;
          font-size: 12px;
        }
        .settings-footer {
          grid-column: 2;
          padding: 12px 22px;
          border-top: 1px solid var(--line);
          display: flex;
          justify-content: flex-end;
        }
        .primary-btn {
          background: var(--accent);
          color: #04140d;
          border: none;
          font-weight: 600;
          font-size: 12.5px;
          padding: 8px 18px;
          border-radius: var(--radius-sm);
        }
      `}</style>
    </div>
  );
}
