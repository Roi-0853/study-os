// ============================================
// lib.rs — Tauri Backend
// ============================================

// Ders + soru sayısını storage/lessons.json dosyasına ekler
#[tauri::command]
fn add_lesson(subject: String, count: u32) -> Result<String, String> {
    use std::io::Write;
    use std::path::Path;

    // Proje kökündeki storage klasörü
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("proje kökü bulunamadı")?
        .join("storage");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join("lessons.json");

    // Mevcut veriyi oku (yoksa boş şablon)
    let mut data: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).unwrap_or(serde_json::json!({ "daily_logs": [] }))
    } else {
        serde_json::json!({ "daily_logs": [] })
    };

    // NOT: chrono crate'i eklendiğinde gerçek tarih kullanın:
    // chrono::Local::now().format("%Y-%m-%d").to_string()
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today = today.as_str();

    let logs = data["daily_logs"]
        .as_array_mut()
        .ok_or("daily_logs bulunamadı")?;

    // Bugünün kaydını bul veya oluştur
    match logs.iter_mut().find(|l| l["date"] == today) {
        Some(log) => {
            let lessons = log["lessons"]
                .as_array_mut()
                .ok_or("lessons bulunamadı")?;

            // Ders zaten varsa sayıyı artır, yoksa yeni ekle
            match lessons.iter_mut().find(|l| l["subject"] == subject) {
                Some(l) => {
                    let current = l["count"].as_u64().unwrap_or(0);
                    l["count"] = serde_json::json!(current + count as u64);
                }
                None => {
                    lessons.push(serde_json::json!({
                        "subject": subject,
                        "count": count
                    }));
                }
            }

            // Toplamı yeniden hesapla
            let total: u64 = lessons
                .iter()
                .map(|l| l["count"].as_u64().unwrap_or(0))
                .sum();
            log["total"] = serde_json::json!(total);
        }
        None => {
            logs.push(serde_json::json!({
                "date": today,
                "lessons": [{ "subject": subject, "count": count }],
                "total": count
            }));
        }
    }

    let text = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;

    Ok(path.display().to_string())
}

// Frontend'in veriyi çekmesi için
#[tauri::command]
fn get_lessons() -> Result<serde_json::Value, String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("proje kökü bulunamadı")?
        .join("storage")
        .join("lessons.json");

    if !path.exists() {
        return Ok(serde_json::json!({ "daily_logs": [] }));
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    Ok(data)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_lesson, get_lessons])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}