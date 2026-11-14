//! Managed provider subprocess protocol and chat adapter.
use crate::model::{ChatModel, Delta, Msg, Stop, StopReason, TurnRequest, TurnResult, Usage};
use craft_anything_evidence::{ErrorCode, ToolError};
use craft_anything_platform::process::{Process, StdinMode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Snapshot {
    pub providers: Vec<ProviderInfo>,
    pub models: Vec<ModelInfo>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub login_available: bool,
    pub authenticated: bool,
    pub disabled_reason: Option<String>,
    pub risk_warning: Option<String>,
    pub auth_kind: Option<String>,
    pub accounts: Vec<AccountInfo>,
    pub usage_error: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct AccountInfo {
    pub id: u64,
    pub label: String,
    pub detail: Option<String>,
    pub kind: String,
    pub selected: bool,
    pub in_use: bool,
    pub state: String,
    pub limits: Vec<UsageLimit>,
    pub fetched_at: Option<u64>,
    pub credits: Option<String>,
    pub usage_error: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct UsageLimit {
    pub id: String,
    pub label: String,
    pub used_fraction: Option<f64>,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub unit: Option<String>,
    pub window_id: Option<String>,
    pub resets_at: Option<u64>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ModelInfo {
    pub provider_id: String,
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub reasoning: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoginEvent {
    OpenUrl { url: String, instructions: Option<String> },
    Input { request_id: String, title: String, placeholder: Option<String>, secret: bool },
    Confirm { request_id: String, title: String, message: String },
    Progress { message: String },
}
pub struct LoginResponse {
    pub request_id: String,
    pub value: Option<String>,
    pub accepted: Option<bool>,
    pub cancelled: bool,
}
pub enum Action {
    Snapshot { refresh: bool },
    Login { provider_id: String },
    SetKey { provider_id: String, key: String },
    SelectAccount { provider_id: String, credential_id: Option<u64> },
    RemoveAccount { provider_id: String, credential_id: u64 },
    Models { provider_id: String },
}
pub enum Event {
    Login(LoginEvent),
    Finished(Result<Snapshot, ToolError>),
    Models(Result<Vec<ModelInfo>, ToolError>),
}
#[derive(Clone)]
pub struct ProviderClient {
    data_dir: PathBuf,
    #[cfg(test)]
    helper_override: Option<PathBuf>,
}
pub struct Task {
    events: mpsc::Receiver<Event>,
    input: mpsc::Sender<Value>,
    cancelled: Arc<AtomicBool>,
}
impl Task {
    pub fn try_event(&mut self) -> Option<Event> {
        loop {
            let event = self.events.try_recv().ok()?;
            if !self.cancelled.load(Ordering::SeqCst) {
                return Some(event);
            }
            match event {
                Event::Login(_) => continue,
                Event::Finished(_) => return Some(Event::Finished(Err(ToolError::cancelled()))),
                Event::Models(_) => return Some(Event::Models(Err(ToolError::cancelled()))),
            }
        }
    }
    pub fn respond(&self, response: LoginResponse) -> Result<(), ToolError> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(ToolError::cancelled());
        }
        self.input.send(json!({"command":"respond","request_id":response.request_id,"value":response.value,"accepted":response.accepted,"cancelled":response.cancelled})).map_err(|_| ToolError::unavailable("Provider task has ended."))
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
impl Drop for Task {
    fn drop(&mut self) {
        self.cancel();
    }
}
fn failure(message: &str) -> ToolError {
    ToolError::new(ErrorCode::ProgramFailed, message)
}
impl ProviderClient {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            #[cfg(test)]
            helper_override: None,
        }
    }
    fn helper(&self) -> Result<PathBuf, ToolError> {
        #[cfg(test)]
        if let Some(path) = &self.helper_override {
            return Ok(path.clone());
        }
        if let Some(path) = std::env::var_os("OMCA_PROVIDER_RUNTIME") {
            let p = PathBuf::from(path);
            if !p.is_absolute() {
                return Err(ToolError::invalid("Provider runtime override must be an absolute path."));
            }
            return Ok(p);
        }
        let exe = std::env::current_exe().map_err(|_| failure("Cannot locate the provider runtime."))?;
        Ok(exe.with_file_name(if cfg!(windows) { "omca-provider-runtime.exe" } else { "omca-provider-runtime" }))
    }
    fn run(
        &self,
        mut request: Value,
        input: mpsc::Receiver<Value>,
        cancelled: Arc<dyn Fn() -> bool + Send + Sync>,
        mut event: impl FnMut(Value) -> Result<(), ToolError>,
    ) -> Result<Value, ToolError> {
        request["id"] = json!("1");
        request["version"] = json!(1);
        let (tx, rx) = mpsc::channel();
        let cancel_process = cancelled.clone();
        let mut process = Process::new(self.helper()?)
            .env_clear()
            .env("OMCA_PROVIDER_DATA_DIR", &self.data_dir)
            .stdin(StdinMode::Piped)
            .timeout(Duration::from_secs(3600))
            .stdout_cap(0)
            .stderr_cap(0)
            .cancel_when(move || cancel_process())
            .on_stdout_line(move |line| {
                let _ = tx.send(line.to_owned());
            });
        for key in [
            "PATH",
            "SystemRoot",
            "WINDIR",
            "TEMP",
            "TMP",
            "TMPDIR",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ] {
            if let Some(v) = std::env::var_os(key) {
                process = process.env(key, v);
            }
        }
        let mut child = process.spawn().map_err(|_| failure("Could not start the provider runtime."))?;
        let mut stdin = child.take_stdin().ok_or_else(|| failure("Provider runtime input unavailable."))?;
        let send = |pipe: &mut std::process::ChildStdin, value: &Value| -> Result<(), ToolError> {
            serde_json::to_writer(&mut *pipe, value).map_err(|_| failure("Could not send provider request."))?;
            pipe.write_all(b"\n").and_then(|_| pipe.flush()).map_err(|_| failure("Could not send provider request."))
        };
        send(&mut stdin, &request)?;
        drop(request);
        loop {
            if cancelled() {
                let _ = send(&mut stdin, &json!({"command":"cancel"}));
                child.kill();
                return Err(ToolError::cancelled());
            }
            while let Ok(value) = input.try_recv() {
                send(&mut stdin, &value)?;
            }
            let line = match rx.recv_timeout(Duration::from_millis(30)) {
                Ok(line) => line,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(failure("Provider runtime ended without a result.")),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if child.try_result().is_none() {
                        continue;
                    }
                    rx.try_recv().map_err(|_| failure("Provider runtime ended without a result."))?
                }
            };
            if cancelled() {
                return Err(ToolError::cancelled());
            }
            let value: Value = serde_json::from_str(&line).map_err(|_| failure("Invalid provider runtime response."))?;
            if value.get("id").and_then(Value::as_str) != Some("1") {
                return Err(failure("Invalid provider runtime response identifier."));
            }
            match value.get("kind").and_then(Value::as_str) {
                Some("result") => return value.get("data").cloned().ok_or_else(|| failure("Provider runtime returned no result.")),
                Some("error") => {
                    let text = value
                        .get("error")
                        .and_then(Value::as_str)
                        .filter(|s| !s.trim().is_empty())
                        .ok_or_else(|| failure("Invalid provider runtime error."))?;
                    let text: String = text.chars().filter(|c| !c.is_control() || *c == '\n').take(1000).collect();
                    return Err(failure(&text));
                }
                Some("login") | Some("delta") => event(value)?,
                _ => return Err(failure("Invalid provider runtime event.")),
            }
        }
    }
    pub fn snapshot(&self, refresh: bool) -> Result<Snapshot, ToolError> {
        let (_tx, rx) = mpsc::channel();
        let value =
            self.run(json!({"command":"snapshot","refresh":refresh}), rx, Arc::new(|| false), |_| Err(failure("Unexpected provider runtime event.")))?;
        serde_json::from_value(value).map_err(|_| failure("Invalid provider snapshot."))
    }
    pub fn start(&self, action: Action) -> Task {
        let (event_tx, events) = mpsc::channel();
        let (input, rx) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let client = self.clone();
        std::thread::spawn(move || {
            let (request, models) = match action {
                Action::Snapshot { refresh } => (json!({"command":"snapshot","refresh":refresh}), false),
                Action::Login { provider_id } => (json!({"command":"login","provider_id":provider_id}), false),
                Action::SetKey { provider_id, key } => (json!({"command":"set_key","provider_id":provider_id,"key":key}), false),
                Action::SelectAccount { provider_id, credential_id } => {
                    (json!({"command":"select_account","provider_id":provider_id,"credential_id":credential_id}), false)
                }
                Action::RemoveAccount { provider_id, credential_id } => {
                    (json!({"command":"remove_account","provider_id":provider_id,"credential_id":credential_id}), false)
                }
                Action::Models { provider_id } => (json!({"command":"models","provider_id":provider_id}), true),
            };
            let result = client.run(request, rx, Arc::new(move || flag.load(Ordering::SeqCst)), |v| {
                if v.get("kind").and_then(Value::as_str) != Some("login") {
                    return Err(failure("Unexpected provider runtime event."));
                }
                let login = serde_json::from_value::<LoginEvent>(v.get("event").cloned().ok_or_else(|| failure("Invalid provider login event."))?)
                    .map_err(|_| failure("Invalid provider login event."))?;
                event_tx.send(Event::Login(login)).map_err(|_| ToolError::cancelled())
            });
            let event = if models {
                Event::Models(result.and_then(|v| serde_json::from_value(v).map_err(|_| failure("Invalid provider model list."))))
            } else {
                Event::Finished(result.and_then(|v| serde_json::from_value(v).map_err(|_| failure("Invalid provider snapshot."))))
            };
            let _ = event_tx.send(event);
        });
        Task { events, input, cancelled }
    }
    pub(crate) fn models(&self, provider_id: &str) -> Result<Vec<ModelInfo>, ToolError> {
        let (_tx, rx) = mpsc::channel();
        let v =
            self.run(json!({"command":"models","provider_id":provider_id}), rx, Arc::new(|| false), |_| Err(failure("Unexpected provider runtime event.")))?;
        serde_json::from_value(v).map_err(|_| failure("Invalid provider model list."))
    }
}
#[derive(Deserialize)]
struct WireTurn {
    message: Msg,
    stop: StopReason,
    usage: Usage,
    model: String,
    follow_up: Option<String>,
}
pub(crate) struct ManagedModel {
    pub client: ProviderClient,
    pub provider_id: String,
    pub model_id: String,
    pub session_id: String,
}
impl ChatModel for ManagedModel {
    fn turn(&mut self, req: &TurnRequest<'_>, on_delta: &mut dyn FnMut(Delta), stop: &Stop) -> Result<TurnResult, ToolError> {
        let tools: Vec<_> = req.tools.iter().map(|t| json!({"name":t.name,"description":t.description,"schema":t.schema})).collect();
        let request = json!({"command":"turn","provider_id":self.provider_id,"model_id":self.model_id,"session_id":self.session_id,"request":{"system":req.system,"messages":req.messages,"tools":tools,"effort":req.effort,"max_tokens":req.max_tokens}});
        let (_tx, rx) = mpsc::channel();
        let stop = stop.clone();
        let v = self.client.run(request, rx, Arc::new(move || stop.is_stopped()), |v| {
            if v.get("kind").and_then(Value::as_str) != Some("delta") {
                return Err(failure("Unexpected provider runtime event."));
            }
            let delta = v.get("delta").ok_or_else(|| failure("Invalid provider delta."))?;
            let text = delta.get("text").and_then(Value::as_str).ok_or_else(|| failure("Invalid provider delta."))?;
            let delta = match delta.get("type").and_then(Value::as_str) {
                Some("text") => Delta::Text(text.to_owned()),
                Some("thinking") => Delta::Thinking(text.to_owned()),
                _ => return Err(failure("Invalid provider delta.")),
            };
            on_delta(delta);
            Ok(())
        })?;
        let r: WireTurn = serde_json::from_value(v).map_err(|_| failure("Invalid provider turn result."))?;
        Ok(TurnResult { message: r.message, stop: r.stop, usage: r.usage, model: r.model, follow_up: r.follow_up })
    }
    fn model(&self) -> String {
        self.model_id.clone()
    }
    fn context_window(&self) -> Option<crate::window::Window> {
        self.client
            .models(&self.provider_id)
            .ok()?
            .into_iter()
            .find(|m| m.id == self.model_id)?
            .context_window
            .map(|tokens| crate::window::Window { tokens, source: "provider catalogue" })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::model::{Block, Effort, Speaker};
    use std::os::unix::fs::PermissionsExt;
    use std::time::Instant;

    fn client(script: &str) -> std::io::Result<(tempfile::TempDir, ProviderClient)> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("helper");
        std::fs::write(&path, format!("#!/bin/sh\n{script}\n"))?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        let client = ProviderClient { data_dir: dir.path().join("providers"), helper_override: Some(path) };
        Ok((dir, client))
    }

    #[test]
    fn malformed_protocol_fails_without_exposing_output() {
        let (_dir, client) = client("read line; echo 'secret-not-json'").unwrap();
        let error = client.snapshot(false).unwrap_err();
        assert!(!error.message.contains("secret-not-json"));
    }

    #[test]
    fn provider_error_is_visible() {
        let (_dir, client) = client("read line; echo '{\"id\":\"1\",\"kind\":\"error\",\"error\":\"Provider disabled\"}'").unwrap();
        assert_eq!(client.snapshot(false).unwrap_err().message, "Provider disabled");
    }

    #[test]
    fn malformed_login_fails_instead_of_waiting() {
        let (_dir, client) = client("read line; echo '{\"id\":\"1\",\"kind\":\"login\",\"event\":{\"type\":\"confirm\"}}'; read line").unwrap();
        let mut task = client.start(Action::Login { provider_id: "test".into() });
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline);
            match task.try_event() {
                Some(Event::Finished(Err(error))) => {
                    assert!(error.message.contains("login event"));
                    break;
                }
                Some(_) => panic!("invalid event was delivered"),
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    #[test]
    fn cancellation_discards_queued_events_and_results() {
        let (sender, events) = mpsc::channel();
        let (input, _receiver) = mpsc::channel();
        let mut task = Task { events, input, cancelled: Arc::new(AtomicBool::new(false)) };
        assert!(sender.send(Event::Login(LoginEvent::Progress { message: "late".into() })).is_ok());
        assert!(sender.send(Event::Finished(Ok(Snapshot::default()))).is_ok());
        task.cancel();
        assert!(matches!(task.try_event(), Some(Event::Finished(Err(_)))));
        assert!(task.try_event().is_none());
    }

    #[test]
    fn malformed_delta_fails_the_turn() {
        let (_dir, client) = client("read line; echo '{\"id\":\"1\",\"kind\":\"delta\",\"delta\":{\"type\":\"text\"}}'; read line").unwrap();
        let mut model = ManagedModel { client, provider_id: "provider".into(), model_id: "test".into(), session_id: "session".into() };
        let request = TurnRequest { system: "", messages: &[], tools: &[], effort: Effort::High, max_tokens: 100 };
        let error = model.turn(&request, &mut |_| {}, &Stop::default()).unwrap_err();
        assert!(error.message.contains("delta"));
    }

    #[test]
    fn cancelling_interactive_task_finishes() {
        let (_dir, client) = client(
            "read line; echo '{\"id\":\"1\",\"kind\":\"login\",\"event\":{\"type\":\"confirm\",\"request_id\":\"p\",\"title\":\"Risk\",\"message\":\"Confirm\"}}'; read line",
        ).unwrap();
        let mut task = client.start(Action::Login { provider_id: "test".into() });
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline);
            if matches!(task.try_event(), Some(Event::Login(LoginEvent::Confirm { .. }))) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        task.cancel();
        loop {
            assert!(Instant::now() < deadline);
            if let Some(Event::Finished(result)) = task.try_event() {
                assert!(result.is_err());
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn managed_turn_preserves_tool_use_and_results() {
        let (_dir, client) = client(
            r#"read line
case "$line" in *'"tool_result"'*) ;; *) exit 1 ;; esac
case "$line" in *'"call-1"'*) ;; *) exit 1 ;; esac
echo '{"id":"1","kind":"result","data":{"message":{"role":"assistant","blocks":[{"type":"tool_use","id":"call-2","name":"lookup","input":{"query":"next"}},{"type":"tool_extra","tool_use_id":"call-2","extra":{"signature":"signed"}}]},"stop":"tool_use","usage":{"input_tokens":1,"output_tokens":2,"cache_read_tokens":0,"cache_write_tokens":0},"model":"test"}}'"#,
        ).unwrap();
        let mut model = ManagedModel { client, provider_id: "provider".into(), model_id: "test".into(), session_id: "session".into() };
        let messages = vec![
            Msg { role: Speaker::Assistant, blocks: vec![Block::ToolUse { id: "call-1".into(), name: "lookup".into(), input: json!({}) }] },
            Msg { role: Speaker::User, blocks: vec![Block::ToolResult { tool_use_id: "call-1".into(), content: "answer".into(), is_error: false }] },
        ];
        let request = TurnRequest { system: "", messages: &messages, tools: &[], effort: Effort::High, max_tokens: 100 };
        let result = model.turn(&request, &mut |_| {}, &Stop::default()).unwrap();
        assert_eq!(result.stop, StopReason::ToolUse);
        assert!(matches!(&result.message.blocks[1], Block::ToolExtra { tool_use_id, extra } if tool_use_id == "call-2" && extra["signature"] == "signed"));
    }
}
