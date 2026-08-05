use crate::agent::abort::ABORT_FLAG;
#[cfg(not(target_arch = "wasm32"))]
use crate::agent::platform_harmonyos::HarmonyOSHost;
#[cfg(not(target_arch = "wasm32"))]
use crate::sandbox::validate::STREAM_CB;
use crate::types::message::{AgentRequest, AgentResponse};
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

/// Global agent configuration injected by the host (configure action).
static AGENT_CONFIG: RwLock<serde_json::Value> =
    RwLock::new(serde_json::Value::Null);

/// WASM virtual filesystem for RAG and search operations.
/// Key: file path, Value: file content
#[cfg(target_arch = "wasm32")]
static VFS: std::sync::LazyLock<RwLock<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(|| RwLock::new(std::collections::HashMap::new()));

pub fn get_config() -> serde_json::Value {
    AGENT_CONFIG.read().unwrap().clone()
}

/// Write content to path only if the file does not already exist.
/// Used to seed defaults on first launch without overwriting user changes.
fn write_default_if_missing(unify_root: &str, file_name: &str, content: Option<&str>) {
    if let Some(data) = content {
        if !data.is_empty() {
            let path = format!("{}/{}", unify_root, file_name);
            if !std::path::Path::new(&path).exists() {
                if let Err(e) = std::fs::write(&path, data) {
                    eprintln!("[json_router] ERROR: write_default '{}' failed: {}", path, e);
                }
            }
        }
    }
}

pub fn dispatch(action: &str, args_json: &str) -> String {
    let request: Result<AgentRequest, _> = serde_json::from_str(args_json);

    let response = match action {
        "ping" => AgentResponse {
            status: "ok".into(),
            message: Some("pong".into()),
            error: None,
        },
        "init_session" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let files_dir = args["files_dir"].as_str().unwrap_or("");
                    // Ensure .unify/ root directory exists
                    let unify_root = format!("{}/.unify", files_dir);
                    if let Err(e) = std::fs::create_dir_all(&unify_root) {
                        eprintln!("[json_router] ERROR: create_dir_all '{}' failed: {}", unify_root, e);
                    }
                    // Create workspace subdirectory for AI file operations
                    if let Err(e) = std::fs::create_dir_all(&format!("{}/workspace", unify_root)) {
                        eprintln!("[json_router] ERROR: create_dir_all '{}/workspace' failed: {}", unify_root, e);
                    }
                    eprintln!("[json_router] .unify root: {}", unify_root);
                    // Write README to explain what .unify/ is
                    let readme_path = format!("{}/README.md", unify_root);
                    if !std::path::Path::new(&readme_path).exists() {
                        if let Err(e) = std::fs::write(&readme_path,
                            "# .unify/ — HmosAgent Data Directory\n\n\
                             All AI Agent configuration, skills, plugins, sessions, and memory data live here.\n\n\
                             - `config.json` — LLM API configuration (api_key, provider, model, base_url)\n\
                             - `providers.json` — AI provider/model registry\n\
                             - `themes.json` — UI theme definitions\n\
                             - `skills/` — Installed agent skills (one subdirectory per skill, each with a `SKILL.md`)\n\
                             - `plugins/` — Installed plugins (one subdirectory per plugin, each with a `plugin.json`)\n\
                             - `sessions/` — Saved conversation sessions\n\
                             - `memory/` — Agent memory storage\n\
                             - `workspace/` — File workspace for AI read/write operations\n"
                        ) {
                            eprintln!("[json_router] ERROR: write README.md '{}' failed: {}", readme_path, e);
                        }
                    }
                    // Write default config files from host-provided defaults (rawfile snapshots)
                    // Only write if the file doesn't already exist (don't overwrite user edits).
                    write_default_if_missing(&unify_root, "config.json", args["defaults"]["config"].as_str());
                    write_default_if_missing(&unify_root, "themes.json", args["defaults"]["themes"].as_str());
                    write_default_if_missing(&unify_root, "providers.json", args["defaults"]["providers"].as_str());
                    // Initialize subsystems
                    crate::agent::session::init_session_manager(files_dir);
                    crate::agent::memory::init_memory_store(&format!("{}/.unify/memory", files_dir));
                    let skills_dir = format!("{}/.unify/skills", files_dir);
                    let skills_path = std::path::Path::new(&skills_dir);
                    // Seed bundled skills on first launch only (no-op if already seeded)
                    let _ = crate::agent::skill_loader::extract_embedded_skills(skills_path);
                    // Clear any stale dynamic entries, then scan disk
                    {
                        let mut registry = crate::agent::skills::SKILLS.write().unwrap();
                        registry.clear_dynamic();
                        // Best-effort scan during init: if the skills directory doesn't
                        // exist yet or is unreadable, we silently continue with an empty
                        // dynamic registry (user can import skills later via reload_imports,
                        // which uses strict error reporting).
                        if let Ok(found) = crate::agent::skill_loader::FileSystemSkillLoader::scan(skills_path) {
                            for skill in found {
                                registry.register_dynamic(skill);
                            }
                        }
                    }
                    let dyn_count = crate::agent::skills::SKILLS.read().unwrap().dynamic_count();
                    let plugins_dir = format!("{}/.unify/plugins", files_dir);
                    crate::agent::plugins::init_plugin_loader(std::path::Path::new(&plugins_dir));
                    AgentResponse {
                        status: "ok".into(),
                        message: Some(format!("Session initialized ({} dynamic skills)", dyn_count)),
                        error: None,
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "get_registered_skills" => {
            let skills = crate::agent::skills::SKILLS.read().unwrap();
            let all = skills.get_all();
            AgentResponse {
                status: "ok".into(),
                message: Some(serde_json::to_string(&all).unwrap_or_default()),
                error: None,
            }
        }
        "load_session" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let session_id = args["session_id"].as_str().unwrap_or("");
                    if session_id.is_empty() {
                        AgentResponse {
                            status: "error".into(),
                            message: None,
                            error: Some("load_session requires session_id".into()),
                        }
                    } else {
                        let mgr = crate::agent::session::SESSION_MGR.read().unwrap();
                        match mgr.as_ref() {
                            Some(m) => match m.load_session(session_id) {
                                Ok(s) => AgentResponse {
                                    status: "ok".into(),
                                    message: Some(
                                        serde_json::to_string(&s).unwrap_or_default(),
                                    ),
                                    error: None,
                                },
                                Err(e) => AgentResponse {
                                    status: "error".into(),
                                    message: None,
                                    error: Some(e),
                                },
                            },
                            None => AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some("Session manager not initialized".into()),
                            },
                        }
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "create_session" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let title = args["title"].as_str().unwrap_or("New Session");
                    let mgr = crate::agent::session::SESSION_MGR.read().unwrap();
                    match mgr.as_ref() {
                        Some(m) => match m.create_session(title) {
                            Ok(s) => AgentResponse {
                                status: "ok".into(),
                                message: Some(
                                    serde_json::to_string(&s).unwrap_or_default(),
                                ),
                                error: None,
                            },
                            Err(e) => AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some(e),
                            },
                        },
                        None => AgentResponse {
                            status: "error".into(),
                            message: None,
                            error: Some("Session manager not initialized".into()),
                        },
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "list_sessions" => {
            let mgr = crate::agent::session::SESSION_MGR.read().unwrap();
            match mgr.as_ref() {
                Some(m) => match m.list_sessions() {
                    Ok(list) => AgentResponse {
                        status: "ok".into(),
                        message: Some(serde_json::to_string(&list).unwrap_or_default()),
                        error: None,
                    },
                    Err(e) => AgentResponse {
                        status: "error".into(),
                        message: None,
                        error: Some(e),
                    },
                },
                None => AgentResponse {
                    status: "ok".into(),
                    message: Some("[]".into()),
                    error: None,
                },
            }
        }
        "delete_session" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let session_id = args["session_id"].as_str().unwrap_or("");
                    if session_id.is_empty() {
                        AgentResponse {
                            status: "error".into(),
                            message: None,
                            error: Some("delete_session requires session_id".into()),
                        }
                    } else {
                        let mgr = crate::agent::session::SESSION_MGR.read().unwrap();
                        match mgr.as_ref() {
                            Some(m) => match m.delete_session(session_id) {
                                Ok(()) => AgentResponse {
                                    status: "ok".into(),
                                    message: Some("Session deleted".into()),
                                    error: None,
                                },
                                Err(e) => AgentResponse {
                                    status: "error".into(),
                                    message: None,
                                    error: Some(e),
                                },
                            },
                            None => AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some("Session manager not initialized".into()),
                            },
                        }
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "save_session" => {
            match serde_json::from_str::<crate::agent::session::Session>(args_json) {
                Ok(session) => {
                    let mgr = crate::agent::session::SESSION_MGR.read().unwrap();
                    match mgr.as_ref() {
                        Some(m) => match m.save_session(&session) {
                            Ok(()) => AgentResponse {
                                status: "ok".into(),
                                message: Some("Session saved".into()),
                                error: None,
                            },
                            Err(e) => AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some(e),
                            },
                        },
                        None => AgentResponse {
                            status: "error".into(),
                            message: None,
                            error: Some("Session manager not initialized".into()),
                        },
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid session: {}", e)),
                },
            }
        }
        "configure" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(cfg) => {
                    // Persist to .unify/config.json for user transparency
                    let sandbox = cfg["sandbox_root"].as_str().unwrap_or(".");
                    let path = format!("{}/.unify/config.json", sandbox);
                    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
                        if let Err(e) = std::fs::write(&path, &json) {
                            eprintln!("[json_router] ERROR: write config to '{}' failed: {}", path, e);
                        } else {
                            eprintln!("[json_router] config saved to: {}", path);
                        }
                    }
                    if let Ok(mut w) = AGENT_CONFIG.write() {
                        *w = cfg;
                    }
                    AgentResponse {
                        status: "ok".into(),
                        message: Some("Configuration stored".into()),
                        error: None,
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid config JSON: {}", e)),
                },
            }
        }
        "update_providers" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let sandbox = args["sandbox_root"].as_str().unwrap_or(".");
                    let data = args["data"].as_str().unwrap_or("");
                    if data.is_empty() {
                        AgentResponse {
                            status: "error".into(),
                            message: None,
                            error: Some("Missing 'data' field for providers".into()),
                        }
                    } else {
                        let path = format!("{}/.unify/providers.json", sandbox);
                        match std::fs::write(&path, data) {
                            Ok(_) => AgentResponse {
                                status: "ok".into(),
                                message: Some("Providers updated".into()),
                                error: None,
                            },
                            Err(e) => AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some(format!("Failed to write providers: {}", e)),
                            },
                        }
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "reload_imports" => {
            match serde_json::from_str::<serde_json::Value>(args_json) {
                Ok(args) => {
                    let files_dir = args["files_dir"].as_str().unwrap_or("");
                    let skills_dir = format!("{}/.unify/skills", files_dir);
                    let path = std::path::Path::new(&skills_dir);
                    // Re-seed bundled skills if needed (no-op if already seeded)
                    let _ = crate::agent::skill_loader::extract_embedded_skills(path);
                    // Clear old dynamic entries, then re-scan disk
                    let skills_loaded;
                    {
                        let mut registry = crate::agent::skills::SKILLS.write().unwrap();
                        registry.clear_dynamic();
                        match crate::agent::skill_loader::FileSystemSkillLoader::scan(path) {
                            Ok(found) => {
                                skills_loaded = found.len();
                                for skill in found {
                                    registry.register_dynamic(skill);
                                }
                            }
                            Err(e) => {
                                return serde_json::to_string(&AgentResponse {
                                    status: "error".into(),
                                    message: None,
                                    error: Some(format!("Skill scan failed: {}", e)),
                                }).unwrap_or_default();
                            }
                        }
                    }
                    // Reload plugins
                    let plugins_dir = format!("{}/.unify/plugins", files_dir);
                    crate::agent::plugins::init_plugin_loader(std::path::Path::new(&plugins_dir));
                    AgentResponse {
                        status: "ok".into(),
                        message: Some(format!(
                            "Reloaded: {} skills, plugins refreshed",
                            skills_loaded
                        )),
                        error: None,
                    }
                }
                Err(e) => AgentResponse {
                    status: "error".into(),
                    message: None,
                    error: Some(format!("Invalid args: {}", e)),
                },
            }
        }
        "abort" => {
            ABORT_FLAG.store(true, Ordering::Relaxed);
            AgentResponse {
                status: "ok".into(),
                message: Some("Generation aborted".into()),
                error: None,
            }
        }
        "list_tasks" => {
            let store = crate::agent::task_state::global_task_store();
            let tasks = store.lock().unwrap().list_tasks();
            let json = serde_json::to_string(&tasks).unwrap_or_else(|_| "[]".into());
            AgentResponse {
                status: "ok".into(),
                message: Some(json),
                error: None,
            }
        }
        "reset_abort" => {
            ABORT_FLAG.store(false, Ordering::Relaxed);
            AgentResponse {
                status: "ok".into(),
                message: Some("Abort flag reset".into()),
                error: None,
            }
        }
        "agent_loop" => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Parse messages directly (bypass AgentRequest untagged ordering issue)
                let args: serde_json::Value = serde_json::from_str(args_json).unwrap_or_default();
                if let Some(msgs_json) = args["messages"].as_array() {
                    let messages: Vec<crate::types::message::ChatMessage> = msgs_json
                        .iter()
                        .filter_map(|m| {
                            Some(crate::types::message::ChatMessage {
                                role: m["role"].as_str()?.to_string(),
                                content: m["content"].as_str()?.to_string(),
                            })
                        })
                        .collect();

                    if !messages.is_empty() {
                        let config = AGENT_CONFIG.read().unwrap().clone();
                        if config.is_null() {
                            return serde_json::to_string(&AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some("Agent not configured. Call 'configure' first.".into()),
                            }).unwrap_or_default();
                        }
                        let sandbox_root = config["sandbox_root"]
                            .as_str()
                            .unwrap_or(".")
                            .to_string();
                        let stream_cb = {
                            let guard = STREAM_CB.lock().unwrap();
                            *guard
                        };

                        if stream_cb.is_none() {
                            return serde_json::to_string(&AgentResponse {
                                status: "error".into(),
                                message: None,
                                error: Some("Stream callback not registered. Did you call init() first?".into()),
                            }).unwrap_or_default();
                        }

                        // Init tool registry if not already done
                        if crate::agent::tool_registry::with_registry(|_| ()).is_none() {
                            crate::agent::tool_registry::init_global_registry(&sandbox_root);
                        }

                        thread::spawn(move || {
                            // Build messages with system prompt
                            let system_prompt = crate::agent::prompt_assembler::assemble_system_prompt(&sandbox_root);
                            let mut internal_messages: Vec<crate::types::message::Message> = vec![
                                crate::types::message::Message {
                                    id: None,
                                    role: "system".into(),
                                    parts: vec![crate::types::message::ContentPart::Text {
                                        text: system_prompt,
                                    }],
                                },
                            ];
                            internal_messages.extend(
                                messages.into_iter().map(|cm| crate::types::message::Message {
                                    id: None,
                                    role: cm.role,
                                    parts: vec![crate::types::message::ContentPart::Text {
                                        text: cm.content,
                                    }],
                                })
                            );

                            // Construct platform host from config
                            let api_key = config["api_key"].as_str().unwrap_or("");
                            let api_base_url = config["base_url"].as_str().unwrap_or("https://api.anthropic.com");

                            let on_text = |text: &str| {
                                if let Some(cb) = stream_cb {
                                    let json = serde_json::json!({"type":"text","text":text});
                                    if let Ok(s) = std::ffi::CString::new(json.to_string()) {
                                        cb(s.as_ptr(), 0);
                                    }
                                }
                            };

                            let on_tool = |name: &str, args: &str| {
                                if let Some(cb) = stream_cb {
                                    let json = serde_json::json!({
                                        "type":"tool_start",
                                        "name": name,
                                        "args": args,
                                    });
                                    if let Ok(s) = std::ffi::CString::new(json.to_string()) {
                                        cb(s.as_ptr(), 3);
                                    }
                                }
                            };

                            let result = crate::agent::tool_registry::with_registry(|r| {
                                let mut msgs = internal_messages;
                                let h = HarmonyOSHost::new(&sandbox_root, api_key, api_base_url);
                                crate::agent::loop_engine::agent_loop_run(
                                    &h,
                                    &config,
                                    &mut msgs,
                                    r,
                                    on_text,
                                    on_tool,
                                )
                            });

                            if let Some(cb) = stream_cb {
                                match result {
                                    Some(Ok(outcome)) => {
                                        let done_json = serde_json::json!({
                                            "type":"done",
                                            "outcome": format!("{:?}", outcome)
                                        });
                                        if let Ok(s) = std::ffi::CString::new(done_json.to_string()) {
                                            cb(s.as_ptr(), 1);
                                        }
                                    }
                                    Some(Err(e)) => {
                                        let err_json = serde_json::json!({
                                            "type":"error",
                                            "text": format!("Agent loop error: {}", e)
                                        });
                                        if let Ok(s) = std::ffi::CString::new(err_json.to_string()) {
                                            cb(s.as_ptr(), 2);
                                        }
                                    }
                                    None => {
                                        let err_json = serde_json::json!({
                                            "type":"error",
                                            "text": "Tool registry not initialized"
                                        });
                                        if let Ok(s) = std::ffi::CString::new(err_json.to_string()) {
                                            cb(s.as_ptr(), 2);
                                        }
                                    }
                                }
                            }
                        });

                        return serde_json::to_string(&AgentResponse {
                            status: "streaming".into(),
                            message: Some("Agent loop started".into()),
                            error: None,
                        }).unwrap_or_default();
                    }
                }
            }
            AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("agent_loop requires messages array".into()),
            }
        }
        "vfs_write_file" => {
            #[cfg(target_arch = "wasm32")]
            {
                if let Ok(req) = request {
                    if let AgentRequest::VfsWrite { path, content } = req {
                        if let Ok(mut vfs) = VFS.write() {
                            let clen = content.len();
                            vfs.insert(path.clone(), content);
                            return serde_json::to_string(&AgentResponse {
                                status: "ok".into(),
                                message: Some(format!("Wrote {} bytes to VFS: {}", clen, path)),
                                error: None,
                            }).unwrap();
                        }
                    }
                }
            }
            AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("vfs_write_file not available on this platform".into()),
            }
        }
        "chat" => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                match request {
                    Ok(AgentRequest::ChatStream { messages }) => {
                        let config = AGENT_CONFIG.read().unwrap().clone();
                        let stream_cb = {
                            let guard = STREAM_CB.lock().unwrap();
                            *guard
                        };

                        thread::spawn(move || {
                            let on_chunk = |chunk_json: &str| {
                                if let Some(cb) = stream_cb {
                                    if let Ok(s) = CString::new(chunk_json) {
                                        cb(s.as_ptr(), 0);
                                    }
                                }
                            };
                            let result = crate::agent::chat::chat_completion_ureq(
                                &config,
                                &messages,
                                on_chunk,
                            );
                            if let Some(cb) = stream_cb {
                                match result {
                                    Ok(()) => {
                                        let done = CString::new("").unwrap();
                                        cb(done.as_ptr(), 1);
                                    }
                                    Err(e) => {
                                        let err_json = format!(
                                            r#"{{"type":"text","text":"{}","is_error":"true"}}"#,
                                            e.replace('"', r#"\""#)
                                        );
                                        if let Ok(s) = CString::new(err_json) {
                                            cb(s.as_ptr(), 2);
                                        }
                                    }
                                }
                            }
                        });

                        return serde_json::to_string(&AgentResponse {
                            status: "streaming".into(),
                            message: Some("Chat streaming started".into()),
                            error: None,
                        }).unwrap_or_default();
                    }
                    _ => {}
                }
            }
            AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("chat requires messages array".into()),
            }
        }
        "test_stream" => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                match request {
                    Ok(AgentRequest::TestStream {
                        chunks,
                        interval_ms,
                    }) => {
                        crate::sandbox::validate::start_stream_sim(chunks, interval_ms);
                        return serde_json::to_string(&AgentResponse {
                            status: "streaming".into(),
                            message: Some("Stream simulation started".into()),
                            error: None,
                        }).unwrap_or_default();
                    }
                    _ => {}
                }
            }
            AgentResponse {
                status: "error".into(),
                message: None,
                error: Some("test_stream requires chunks and interval_ms params".into()),
            }
        },
        _ => AgentResponse {
            status: "error".into(),
            message: None,
            error: Some(format!("Unknown action: {}", action)),
        },
    };

    serde_json::to_string(&response).unwrap_or_else(|e| {
        format!(
            r#"{{"status":"error","error":"JSON serialize failed: {}"}}"#,
            e
        )
    })
}
