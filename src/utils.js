// ============================================
// utils.js — Yardımcı Fonksiyonlar
// ============================================

export function getLastNDays(n) {
  const days = [];
  const today = new Date();
  for (let i = n - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    days.push(d.toISOString().split("T")[0]);
  }
  return days;
}

export async function invokeCmd(cmd, args = {}) {
  const { invoke } = window.__TAURI__.core;
  return await invoke(cmd, args);
}

export function openWindow({ title = "Pencere", content = "" }) {
  const template = document.querySelector("#window-template");
  if (!template) {
    console.error("window-template bulunamadı!");
    return null;
  }

  const clone = template.content.cloneNode(true);
  const win = clone.querySelector(".floating-window");

  win.querySelector(".window-title").textContent = title;
  win.querySelector(".window-content").innerHTML = content;

  win.style.left = 100 + Math.random() * 200 + "px";
  win.style.top = 100 + Math.random() * 100 + "px";

  document.body.appendChild(win);

  // Sürükleme
  const titlebar = win.querySelector(".window-titlebar");
  let isDragging = false;
  let offsetX = 0, offsetY = 0;

  titlebar.addEventListener("mousedown", (e) => {
    isDragging = true;
    const rect = win.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;
  });

  document.addEventListener("mousemove", (e) => {
    if (!isDragging) return;
    win.style.left = e.clientX - offsetX + "px";
    win.style.top = e.clientY - offsetY + "px";
  });

  document.addEventListener("mouseup", () => {
    isDragging = false;
  });

  win.querySelector(".window-close").addEventListener("click", () => win.remove());

  return win;
}