use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub meta: SessionMeta,
    pub messages: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionIndex {
    sessions: Vec<SessionMeta>,
}

pub struct SessionManager {
    base_dir: PathBuf,
}

impl SessionManager {
    pub fn new(files_dir: &str) -> Self {
        let base_dir = PathBuf::from(files_dir).join(".unify/sessions");
        if let Err(e) = fs::create_dir_all(&base_dir) {
            eprintln!("[session] ERROR: create_dir_all '{}' failed: {}", base_dir.display(), e);
        }
        Self { base_dir }
    }

    pub fn create_session(&self, title: &str) -> Result<Session, String> {
        let id = uuid_v4();
        let now = timestamp_now();
        let meta = SessionMeta {
            id: id.clone(),
            title: title.to_string(),
            created_at: now.clone(),
            updated_at: now,
            message_count: 0,
        };
        let session = Session {
            meta: meta.clone(),
            messages: vec![],
        };
        self.save_session_file(&session)?;
        self.add_to_index(&meta)?;
        Ok(session)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionMeta>, String> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("read index: {}", e))?;
        let index: SessionIndex = serde_json::from_str(&data)
            .unwrap_or(SessionIndex { sessions: vec![] });
        Ok(index.sessions)
    }

    pub fn load_session(&self, id: &str) -> Result<Session, String> {
        let path = self.session_path(id);
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("read session {}: {}", id, e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("parse session {}: {}", id, e))
    }

    pub fn delete_session(&self, id: &str) -> Result<(), String> {
        let path = self.session_path(id);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("delete session {}: {}", id, e))?;
        }
        self.remove_from_index(id)?;
        Ok(())
    }

    pub fn save_session(&self, session: &Session) -> Result<(), String> {
        self.save_session_file(session)?;
        self.update_index_meta(&session.meta)?;
        Ok(())
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", id))
    }

    fn index_path(&self) -> PathBuf {
        self.base_dir.join("index.json")
    }

    fn save_session_file(&self, session: &Session) -> Result<(), String> {
        let path = self.session_path(&session.meta.id);
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| format!("serialize session: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("write session: {}", e))
    }

    fn add_to_index(&self, meta: &SessionMeta) -> Result<(), String> {
        let mut index = self.read_index().unwrap_or(SessionIndex { sessions: vec![] });
        index.sessions.push(meta.clone());
        self.write_index(&index)
    }

    fn remove_from_index(&self, id: &str) -> Result<(), String> {
        let mut index = self.read_index().unwrap_or(SessionIndex { sessions: vec![] });
        index.sessions.retain(|s| s.id != id);
        self.write_index(&index)
    }

    fn update_index_meta(&self, meta: &SessionMeta) -> Result<(), String> {
        let mut index = self.read_index().unwrap_or(SessionIndex { sessions: vec![] });
        if let Some(existing) = index.sessions.iter_mut().find(|s| s.id == meta.id) {
            existing.updated_at = meta.updated_at.clone();
            existing.message_count = meta.message_count;
            existing.title = meta.title.clone();
        }
        self.write_index(&index)
    }

    fn read_index(&self) -> Result<SessionIndex, String> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(SessionIndex { sessions: vec![] });
        }
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("read index: {}", e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("parse index: {}", e))
    }

    fn write_index(&self, index: &SessionIndex) -> Result<(), String> {
        let path = self.index_path();
        let json = serde_json::to_string_pretty(index)
            .map_err(|e| format!("serialize index: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("write index: {}", e))
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Simple deterministic ID from timestamp + random suffix for uniqueness
    let rand_part: u64 = ts as u64 ^ 0xDEADBEEF;
    format!("{:x}{:x}", ts, rand_part)
}

fn timestamp_now() -> String {
    // ISO 8601 compact: 2026-05-27T13:45:00
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days_since_1970 = secs / 86400;
    // civil calendar conversion from days since Unix epoch
    let (y, m, d) = civil_from_days(days_since_1970 as i64 + 719468);
    let remaining = secs % 86400;
    let h = remaining / 3600;
    let min = (remaining % 3600) / 60;
    let s = remaining % 60;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, h, min, s)
}

/// Convert days since epoch to (year, month, day)
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Based on Howard Hinnant's algorithm
    let z = days;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Global session manager singleton
pub static SESSION_MGR: RwLock<Option<SessionManager>> = RwLock::new(None);

pub fn init_session_manager(files_dir: &str) {
    let mut mgr = SESSION_MGR.write().unwrap();
    *mgr = Some(SessionManager::new(files_dir));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup(name: &str) -> SessionManager {
        let dir = env::temp_dir().join(format!("hmos_test_sessions_{}", name));
        let _ = fs::remove_dir_all(&dir);
        SessionManager::new(dir.to_str().unwrap())
    }

    #[test]
    fn test_create_and_list() {
        let mgr = setup("cal");
        let s = mgr.create_session("test chat").unwrap();
        assert!(!s.meta.id.is_empty());
        assert_eq!(s.meta.title, "test chat");

        let list = mgr.list_sessions().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, s.meta.id);
    }

    #[test]
    fn test_load_and_delete() {
        let mgr = setup("lad");
        let s = mgr.create_session("load test").unwrap();
        let loaded = mgr.load_session(&s.meta.id).unwrap();
        assert_eq!(loaded.meta.title, "load test");

        mgr.delete_session(&s.meta.id).unwrap();
        let list = mgr.list_sessions().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_save_session() {
        let mgr = setup("ss");
        let mut s = mgr.create_session("save test").unwrap();
        s.messages.push(serde_json::json!({"role":"user","content":"hello"}));
        s.meta.message_count = 1;
        mgr.save_session(&s).unwrap();

        let loaded = mgr.load_session(&s.meta.id).unwrap();
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.meta.message_count, 1);
    }

    #[test]
    fn test_multiple_sessions() {
        let mgr = setup("ms");
        mgr.create_session("a").unwrap();
        mgr.create_session("b").unwrap();
        mgr.create_session("c").unwrap();
        let list = mgr.list_sessions().unwrap();
        assert_eq!(list.len(), 3);
    }
}
