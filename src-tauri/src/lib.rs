// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Ders + soru sayısını proje içindeki storage/lessons.json dosyasına ekler
#[tauri::command]
fn add_lesson(subject: String, count: u32) -> Result<String, String> {
    use std::io::Write;

    // Proje kökündeki storage klasörü: src-tauri/../storage
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("proje kökü bulunamadı")?
        .join("storage");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join("lessons.json");

    // Dosya varsa oku, yoksa boş liste ile başla
    let mut lessons: Vec<serde_json::Value> = if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).unwrap_or_default()
    } else {
        Vec::new()
    };

    lessons.push(serde_json::json!({ "subject": subject, "count": count }));

    let text = serde_json::to_string_pretty(&lessons).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;

    Ok(path.display().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, add_lesson])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
