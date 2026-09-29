// lib.rs — Study-OS Tauri Backend

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use serde::{Deserialize, Serialize};

mod commands;
use commands::{add_lesson, get_lessons};

// Sabitler

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub const VALID_SUBJECTS: [&str; 4] = ["mat", "fizik", "kimya", "biyo"];

const STORE_FILE: &str = "study_os.json";

/// Depo (storage) klasörü adı — proje kökü altında `storage/`
const STORE_DIR: &str = "storage";

/// Depo yolunu geçersiz kılmak için isteğe bağlı ortam değişkeni.
///   STUDY_OS_STORAGE=C:\Users\Mustafa\Desktop\study-os\storage
const STORE_DIR_ENV: &str = "STUDY_OS_STORAGE";

/// Proje kökü: `src-tauri` kapsayıcısının bir üst klasörü.
/// `env!` derleme anında sabitlenir, bu yüzden çalışma zamanında `current_dir()` gerekmez.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

// ------------------------------------------------------------
// Veri Modelleri
// ------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonEntry {
    pub subject: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub date: String,
    pub total: u32,
    pub lessons: Vec<LessonEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub daily_logs: Vec<DailyLog>,
}

fn default_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            daily_logs: Vec::new(),
        }
    }
}

// ------------------------------------------------------------
// Hata Tipi
// ------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("io hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("json hatası: {0}")]
    Json(#[from] serde_json::Error),

    #[error("geçersiz ders: {0}")]
    InvalidSubject(String),

    #[error("geçersiz sayı: {0}")]
    InvalidCount(String),

    #[error("şema sürümü desteklenmiyor: dosya={file}, beklenen<={expected}")]
    UnsupportedVersion { file: u32, expected: u32 },
}

impl Serialize for StoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ------------------------------------------------------------
// Depo Yolu
// ------------------------------------------------------------

pub fn store_path(app: &tauri::AppHandle) -> Result<PathBuf, StoreError> {
    let _ = app;

    let dir = match std::env::var_os(STORE_DIR_ENV) {
        Some(custom) if !custom.is_empty() => PathBuf::from(custom),
        _ => project_root().join(STORE_DIR),
    };

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir.join(STORE_FILE))
}

// ------------------------------------------------------------
// Okuma / Yazma
// ------------------------------------------------------------

pub fn read_store(app: &tauri::AppHandle) -> Result<Store, StoreError> {
    let path = store_path(app)?;

    if !path.exists() {
        return Ok(Store::default());
    }

    let raw = fs::read_to_string(&path)?;

    if raw.trim().is_empty() {
        return Ok(Store::default());
    }

    let store: Store = serde_json::from_str(&raw)?;

    if store.version > CURRENT_SCHEMA_VERSION {
        return Err(StoreError::UnsupportedVersion {
            file: store.version,
            expected: CURRENT_SCHEMA_VERSION,
        });
    }

    Ok(store)
}

pub fn write_store(app: &tauri::AppHandle, mut store: Store) -> Result<(), StoreError> {
    store.version = CURRENT_SCHEMA_VERSION;
    store.daily_logs.sort_by(|a, b| a.date.cmp(&b.date));

    let path = store_path(app)?;
    let json = serde_json::to_string_pretty(&store)?;

    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;

    Ok(())
}

// ------------------------------------------------------------
// Yardımcılar
// ------------------------------------------------------------

pub fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn validate_subject(subject: &str) -> Result<(), StoreError> {
    if VALID_SUBJECTS.contains(&subject) {
        Ok(())
    } else {
        Err(StoreError::InvalidSubject(subject.to_string()))
    }
}

// ------------------------------------------------------------
// Tauri State
// ------------------------------------------------------------

pub struct AppState {
    pub lock: Mutex<()>,
}

// ------------------------------------------------------------
// Uygulama Girişi
// ------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            lock: Mutex::new(()),
        })
        .invoke_handler(tauri::generate_handler![get_lessons, add_lesson])
        .setup(|app| {
            let handle = app.handle().clone();
            if let Ok(path) = store_path(&handle) {
                if !path.exists() {
                    let _ = write_store(&handle, Store::default());
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması başlatılamadı");
}