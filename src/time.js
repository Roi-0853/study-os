// time.js — Saat


export function startClock() {
  const clockTime = document.querySelector(".clock-time");
  const clockDate = document.querySelector(".clock-date");
  if (!clockTime || !clockDate) return;

  function update() {
    const now = new Date();
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    clockTime.textContent = `${h}:${m}`;

    const opts = { month: "short", day: "numeric" };
    clockDate.textContent = now.toLocaleDateString("en-US", opts).toLowerCase();
  }

  update();
  setInterval(update, 1000);
}