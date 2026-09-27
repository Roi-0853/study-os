// ============================================================
// lib.rs — Study-OS Tauri Backend
// ============================================================

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

// ------------------------------------------------------------
// Sabitler
// ------------------------------------------------------------

pub const CURRENT_SCHEMA_VERSION: u32 = 0;

const VALID_SUBJECTS: [&str; 4] = ["mat", "fizik", "kimya", "biyo"];

const STORE_FILE: &str = "study_os.json";

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

fn store_path(app: &tauri::AppHandle) -> Result<PathBuf, StoreError> {
    let dir = app.path().app_data_dir().map_err(|e| {
        StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir.join(STORE_FILE))
}

// ------------------------------------------------------------
// Okuma / Yazma
// ------------------------------------------------------------

fn read_store(app: &tauri::AppHandle) -> Result<Store, StoreError> {
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

fn write_store(app: &tauri::AppHandle, mut store: Store) -> Result<(), StoreError> {
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

fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn validate_subject(subject: &str) -> Result<(), StoreError> {
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
// Tauri Komutları  (HER BİRİ SADECE 1 KEZ)
// ------------------------------------------------------------

#[tauri::command]
pub fn get_lessons(app: tauri::AppHandle) -> Result<Store, StoreError> {
    read_store(&app)
}

#[tauri::command]
pub fn add_lesson(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    subject: String,
    count: u32,
) -> Result<Store, StoreError> {
    let _guard = state.lock.lock().map_err(|_| {
        StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "state kilidi zehirlendi",
        ))
    })?;

    if count == 0 {
        return Err(StoreError::InvalidCount(count.to_string()));
    }
    validate_subject(&subject)?;

    let mut store = read_store(&app)?;
    let today = today_str();

    let log = match store.daily_logs.iter_mut().find(|l| l.date == today) {
        Some(l) => l,
        None => {
            store.daily_logs.push(DailyLog {
                date: today.clone(),
                total: 0,
                lessons: Vec::new(),
            });
            store.daily_logs.last_mut().unwrap()
        }
    };

    match log.lessons.iter_mut().find(|e| e.subject == subject) {
        Some(entry) => entry.count += count,
        None => log.lessons.push(LessonEntry {
            subject: subject.clone(),
            count,
        }),
    }

    log.total = log.lessons.iter().map(|e| e.count).sum();

    log.lessons.sort_by_key(|e| {
        VALID_SUBJECTS
            .iter()
            .position(|s| *s == e.subject)
            .unwrap_or(usize::MAX)
    });

    write_store(&app, store.clone())?;

    Ok(store)
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