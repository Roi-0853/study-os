// ============================================
// main.js — Giriş Noktası
// ============================================

import { startClock } from "./time.js";
import { openWindow, invokeCmd } from "./utils.js";
import { refreshDashboard } from "./graphics.js";

function setupAddLesson() {
  const btn = document.querySelector("#addlesson");
  if (!btn) return;

  btn.addEventListener("click", () => {
    const win = openWindow({
      title: "Ders Ekle",
      content: `
        <form id="lesson-form">
          <label>Ders
            <select name="subject">
              <option value="mat">Matematik</option>
              <option value="fizik">Fizik</option>
              <option value="kimya">Kimya</option>
              <option value="biyo">Biyoloji</option>
            </select>
          </label>
          <label>Soru Sayısı
            <input name="count" type="number" min="1" value="10">
          </label>
          <button type="submit">Ekle</button>
        </form>
      `,
    });

    if (!win) return;

    win.querySelector("#lesson-form").addEventListener("submit", async (e) => {
      e.preventDefault();
      const form = new FormData(e.target);
      const subject = form.get("subject");
      const count = parseInt(form.get("count"), 10);

      try {
        await invokeCmd("add_lesson", { subject, count });
        win.remove();
        await refreshDashboard();
      } catch (err) {
        alert("Hata: " + err);
      }
    });
  });
}

document.addEventListener("DOMContentLoaded", () => {
  startClock();
  setupAddLesson();
  refreshDashboard();
});