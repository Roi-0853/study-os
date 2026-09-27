// storage.js veya main.js
function openWindow({ title = "Pencere", content = "" }) {
  const template = document.querySelector("#window-template");
  const clone = template.content.cloneNode(true);
  const win = clone.querySelector(".floating-window");

  // Başlık ve içerik
  win.querySelector(".window-title").textContent = title;
  win.querySelector(".window-content").innerHTML = content;

  // Rastgele konum (her pencere üst üste gelmesin)
  win.style.left = 100 + Math.random() * 200 + "px";
  win.style.top  = 100 + Math.random() * 100 + "px";

  document.body.appendChild(win);

  // --- Sürükleme ---
  const titlebar = win.querySelector(".window-titlebar");
  let isDragging = false;
  let offsetX = 0, offsetY = 0;

  titlebar.addEventListener("mousedown", (e) => {
    isDragging = true;
    const rect = win.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;
    document.body.style.userSelect = "none"; // sürüklerken metin seçme
  });

  document.addEventListener("mousemove", (e) => {
    if (!isDragging) return;
    win.style.left = e.clientX - offsetX + "px";
    win.style.top  = e.clientY - offsetY + "px";
  });

  document.addEventListener("mouseup", () => {
    if (!isDragging) return;
    isDragging = false;
    document.body.style.userSelect = "";
  });

  // --- Kapatma ---
  win.querySelector(".window-close").addEventListener("click", () => {
    win.remove();
  });

  return win; // dışarıdan içerik eklemek istersen
}

const addlesson = document.querySelector("#addlesson");

if (addlesson) {
  addlesson.addEventListener("click", () => {
    openWindow({
      title: "Ders Ekle",
      content: `
        <form id="lesson-form">
          <label>Ders
            <select name="subject">
              <option value="mat">Matematik</option>
              <option value="fizik">Fizik</option>
              <option value="kimya">Kimya</option>
              <option value="biyo">Biyoloji</option>
              <option value="edeb">Edebiyat</option>
            </select>
          </label>
          <label>Soru Sayısı
            <input name="count" type="number" min="1" value="10">
          </label>
          <button type="submit">Ekle</button>
        </form>
      `,
    });

    // Form submit'ini yakala
    document.querySelector("#lesson-form").addEventListener("submit", (e) => {
      e.preventDefault();
      const data = new FormData(e.target);
      const subject = data.get("subject");
      const count = parseInt(data.get("count"), 10);

      // Rust tarafına gönder: storage/lessons.json dosyasına yazar
      const { invoke } = window.__TAURI__.core;
      invoke("add_lesson", { subject, count })
        .then((path) => console.log("Kaydedildi:", path))
        .catch((err) => console.error("Kaydedilemedi:", err));

      e.target.closest(".floating-window").remove();
    });
  });
}

// --- SAAT FONKSİYONU ---
function startClock() {
  const clockTimeElement = document.querySelector('.clock-time');
  const clockDateElement = document.querySelector('.clock-date');

  // Eğer HTML'de bu elementler yoksa fonksiyondan çık
  if (!clockTimeElement || !clockDateElement) return;

  function updateClock() {
    const now = new Date();

    // Saat formatı (09:32 gibi)
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    clockTimeElement.textContent = `${hours}:${minutes}`;

    // Tarih formatı (sep 3 gibi)
    const options = { month: 'short', day: 'numeric' };
    // Türkçe ay isimleri için 'tr-TR' kullanabilirsiniz, 
    // ama görselde İngilizce kısaltma var (sep)
    const dateString = now.toLocaleDateString('en-US', options).toLowerCase();
    clockDateElement.textContent = dateString;
  }

  // İlk çalıştırma
  updateClock();

  // Her saniye güncelle (1000ms)
  setInterval(updateClock, 1000);
}

// DOM yüklendiğinde saati başlat
document.addEventListener('DOMContentLoaded', () => {
  startClock();
});