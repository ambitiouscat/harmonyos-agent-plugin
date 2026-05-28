use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

// ── Task model ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub status: TaskStatus,
    pub blocked_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

// ── Task store ──

pub struct TaskStore {
    tasks: HashMap<String, Task>,
    /// Counter for how many rounds the agent has gone without calling a task tool.
    pub rounds_since_task_tool: AtomicU32,
    /// Flag indicating any task tool was called in the current round.
    task_tool_called_this_round: AtomicBool,
}

pub type SharedTaskStore = Arc<Mutex<TaskStore>>;

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            rounds_since_task_tool: AtomicU32::new(0),
            task_tool_called_this_round: AtomicBool::new(false),
        }
    }

    pub fn shared() -> SharedTaskStore {
        Arc::new(Mutex::new(Self::new()))
    }

    /// Create a new task. Returns error if ID already exists.
    pub fn create_task(&mut self, id: String, content: String, blocked_by: Vec<String>) -> Result<Task, String> {
        if self.tasks.contains_key(&id) {
            return Err(format!("Task '{}' already exists", id));
        }
        let task = Task {
            id: id.clone(),
            content,
            status: TaskStatus::Pending,
            blocked_by: blocked_by.clone(),
            owner: None,
        };
        self.tasks.insert(id, task.clone());
        self.mark_task_tool_called();
        Ok(task)
    }

    /// Update task status and/or content.
    pub fn update_task(
        &mut self,
        id: &str,
        status: Option<TaskStatus>,
        content: Option<String>,
    ) -> Result<Task, String> {
        let result = {
            let task = self
                .tasks
                .get_mut(id)
                .ok_or_else(|| format!("Task '{}' not found", id))?;
            if let Some(s) = status {
                task.status = s;
            }
            if let Some(c) = content {
                task.content = c;
            }
            task.clone()
        }; // mutable borrow of self.tasks ends here
        self.mark_task_tool_called();
        Ok(result)
    }

    /// List all tasks.
    pub fn list_tasks(&self) -> Vec<Task> {
        let mut tasks: Vec<Task> = self.tasks.values().cloned().collect();
        tasks.sort_by(|a, b| a.id.cmp(&b.id));
        tasks
    }

    /// Mark that a task tool was called this round.
    pub fn mark_task_tool_called(&self) {
        self.task_tool_called_this_round.store(true, Ordering::Relaxed);
    }

    /// Called at end of each round. Increments counter if no task tool was called.
    /// Returns true if a nag reminder should be injected (>3 rounds without task tool).
    pub fn end_round_nag(&self) -> Option<String> {
        if self.task_tool_called_this_round.swap(false, Ordering::Relaxed) {
            self.rounds_since_task_tool.store(0, Ordering::Relaxed);
            None
        } else {
            let rounds = self.rounds_since_task_tool.fetch_add(1, Ordering::Relaxed) + 1;
            if rounds >= 3 {
                Some(
                    "<system-reminder>You have not updated your task list in a while. \
                     Consider using task_create or task_update to track your progress.</system-reminder>"
                        .into(),
                )
            } else {
                None
            }
        }
    }

    /// Check if all tasks are completed.
    pub fn all_completed(&self) -> bool {
        !self.tasks.is_empty()
            && self
                .tasks
                .values()
                .all(|t| t.status == TaskStatus::Completed)
    }
}

// ── Global task store ──

static GLOBAL_TASK_STORE: std::sync::LazyLock<SharedTaskStore> =
    std::sync::LazyLock::new(TaskStore::shared);

pub fn global_task_store() -> SharedTaskStore {
    Arc::clone(&GLOBAL_TASK_STORE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list() {
        let store = TaskStore::shared();
        let mut s = store.lock().unwrap();
        s.create_task("1".into(), "Task 1".into(), vec![]).unwrap();
        s.create_task("2".into(), "Task 2".into(), vec!["1".into()])
            .unwrap();

        let tasks = s.list_tasks();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].blocked_by, vec![] as Vec<String>);
        assert_eq!(tasks[1].blocked_by, vec!["1"]);
    }

    #[test]
    fn test_update_status() {
        let store = TaskStore::shared();
        let mut s = store.lock().unwrap();
        s.create_task("1".into(), "Task 1".into(), vec![]).unwrap();
        s.update_task("1", Some(TaskStatus::InProgress), None).unwrap();
        assert_eq!(s.tasks["1"].status, TaskStatus::InProgress);
    }

    #[test]
    fn test_duplicate_id() {
        let store = TaskStore::shared();
        let mut s = store.lock().unwrap();
        s.create_task("1".into(), "T1".into(), vec![]).unwrap();
        let result = s.create_task("1".into(), "T1 again".into(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_nag_reminder() {
        let store = TaskStore::shared();
        // Round 1: no task tool → no nag yet
        assert!(store.lock().unwrap().end_round_nag().is_none());
        // Round 2: no task tool → no nag yet
        assert!(store.lock().unwrap().end_round_nag().is_none());
        // Round 3: no task tool → nag triggers (rounds >= 3)
        let nag = store.lock().unwrap().end_round_nag();
        assert!(nag.is_some());
        assert!(nag.unwrap().contains("task_create"));
    }

    #[test]
    fn test_nag_reset_on_task_tool() {
        let store = TaskStore::shared();
        store.lock().unwrap().end_round_nag(); // 1
        store.lock().unwrap().end_round_nag(); // 2
        store.lock().unwrap().mark_task_tool_called();
        store.lock().unwrap().end_round_nag(); // should reset to 0
        assert!(store.lock().unwrap().end_round_nag().is_none()); // 1
    }

    #[test]
    fn test_all_completed() {
        let store = TaskStore::shared();
        let mut s = store.lock().unwrap();
        s.create_task("1".into(), "T1".into(), vec![]).unwrap();
        assert!(!s.all_completed());
        s.update_task("1", Some(TaskStatus::Completed), None).unwrap();
        assert!(s.all_completed());
    }
}
