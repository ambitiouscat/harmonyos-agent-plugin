use serde_json::Value;

use crate::agent::task_state::{global_task_store, TaskStatus};

/// task_create tool handler.
pub fn task_create_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let id = args["id"]
        .as_str()
        .ok_or_else(|| "Missing 'id' parameter".to_string())?;
    let content = args["content"]
        .as_str()
        .ok_or_else(|| "Missing 'content' parameter".to_string())?;
    let blocked_by: Vec<String> = args["blocked_by"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let arc = global_task_store();
    let mut store = arc.lock().unwrap();
    match store.create_task(id.to_string(), content.to_string(), blocked_by) {
        Ok(task) => Ok(serde_json::to_string(&task).unwrap_or_else(|_| "Task created".into())),
        Err(e) => Err(e),
    }
}

/// task_update tool handler.
pub fn task_update_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let id = args["id"]
        .as_str()
        .ok_or_else(|| "Missing 'id' parameter".to_string())?;
    let status_str = args["status"].as_str();
    let content = args["content"].as_str().map(String::from);

    let status = match status_str {
        Some("pending") => Some(TaskStatus::Pending),
        Some("in_progress") => Some(TaskStatus::InProgress),
        Some("completed") => Some(TaskStatus::Completed),
        Some(s) => return Err(format!("Invalid status '{}'. Use pending/in_progress/completed.", s)),
        None => None,
    };

    let arc = global_task_store();
    let mut store = arc.lock().unwrap();
    match store.update_task(id, status, content) {
        Ok(task) => Ok(serde_json::to_string(&task).unwrap_or_else(|_| "Task updated".into())),
        Err(e) => Err(e),
    }
}

/// task_list tool handler.
pub fn task_list_handler(_args: Value, _sandbox_root: &str) -> Result<String, String> {
    let arc = global_task_store();
    let store = arc.lock().unwrap();
    let tasks = store.list_tasks();
    if tasks.is_empty() {
        Ok("No tasks found.".into())
    } else {
        Ok(serde_json::to_string_pretty(&tasks).unwrap_or_else(|_| "Error serializing tasks".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_create_and_list() {
        let _ = task_create_handler(
            serde_json::json!({"id": "t1", "content": "Do something"}),
            "",
        );
        let result = task_list_handler(serde_json::json!({}), "");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("t1"));
        assert!(output.contains("Do something"));
    }

    #[test]
    fn test_task_update() {
        let _ = task_create_handler(
            serde_json::json!({"id": "t2", "content": "Task 2"}),
            "",
        );
        let result = task_update_handler(
            serde_json::json!({"id": "t2", "status": "in_progress"}),
            "",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_update_invalid_status() {
        let _ = task_create_handler(
            serde_json::json!({"id": "t3", "content": "T3"}),
            "",
        );
        let result = task_update_handler(
            serde_json::json!({"id": "t3", "status": "invalid"}),
            "",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_task_create_missing_id() {
        let result = task_create_handler(serde_json::json!({}), "");
        assert!(result.is_err());
    }
}
