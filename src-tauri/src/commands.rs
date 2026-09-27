use tauri::State;

use crate::{
    read_store, today_str, validate_subject, write_store, AppState, DailyLog, LessonEntry, Store,
    StoreError, VALID_SUBJECTS,
};

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
