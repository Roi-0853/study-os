// graphics.js — Grid + Radar

const Chart = window.Chart;

import { invokeCmd, getLastNDays } from "./utils.js";

let radarChart = null;

//GRID
export async function renderGrid() {
  const container = document.querySelector(".grid-container");
  if (!container) return;

  const data = await invokeCmd("get_lessons");
  const logs = data.daily_logs || [];

  container.innerHTML = "";
  const days = getLastNDays(100);

  days.forEach((date) => {
    const log = logs.find((l) => l.date === date);
    const total = log ? log.total : 0;

    const cell = document.createElement("div");
    cell.classList.add("grid-cell");

    if (total > 100) cell.classList.add("level-3");
    else if (total > 50) cell.classList.add("level-2");
    else if (total > 0) cell.classList.add("level-1");

    cell.title = `${date}: ${total} soru`;
    container.appendChild(cell);
  });
}

//RADAR
export async function renderRadar() {
  const canvas = document.querySelector("#radar-canvas");
  if (!canvas) return;

  const data = await invokeCmd("get_lessons");
  const logs = data.daily_logs || [];

  const totals = { mat: 0, fizik: 0, kimya: 0, biyo: 0 };
  const last30 = getLastNDays(30);

  logs.forEach((log) => {
    if (last30.includes(log.date)) {
      log.lessons.forEach((l) => {
        if (totals[l.subject] !== undefined) totals[l.subject] += l.count;
      });
    }
  });

  const chartData = {
    labels: ["Mat", "Fizik", "Kimya", "Biyo"],
    datasets: [{
      data: [totals.mat, totals.fizik, totals.kimya, totals.biyo],
      backgroundColor: "rgba(122, 255, 149, 0.15)",
      borderColor: "#7AFF95",
      borderWidth: 2,
      pointBackgroundColor: "#7AFF95",
    }],
  };

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: { legend: { display: false } },
    scales: {
      r: {
        beginAtZero: true,
        grid: { color: "#1a3a2a" },
        angleLines: { color: "#1a3a2a" },
        pointLabels: { color: "#7AFF95", font: { size: 11 } },
        ticks: { display: false },
      },
    },
  };

  if (radarChart) {
    radarChart.data = chartData;
    radarChart.update();
  } else {
    radarChart = new Chart(canvas, {
      type: "radar",
      data: chartData,
      options: chartOptions,
    });
  }
}

//YENİLE
export async function refreshDashboard() {
  await Promise.all([renderGrid(), renderRadar()]);
}