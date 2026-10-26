use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Done,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub subtitle: String,
    pub hidden: bool,
    pub cancellable: bool,
    pub status: TaskStatus,
    pub received: u64,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub text: String,
    pub image: String,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            title: "Bottles".to_owned(),
            text: "no message provided".to_owned(),
            image: String::new(),
        }
    }
}

/// Signal names from `bottles/backend/state.py`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Signal {
    ManagerLocalBottlesLoaded,
    ForceStopNetworking,
    RepositoryFetched,
    NetworkStatusChanged,
    GNotification,
    GShowUri,
    TaskAdded,
    TaskRemoved,
    TaskUpdated,
    ProgramStarted,
    ProgramFinished,
    EagleStep,
    EagleFinished,
    DesktopEntryCreated,
}

impl Signal {
    pub const ALL: &'static [Self] = &[
        Self::ManagerLocalBottlesLoaded,
        Self::ForceStopNetworking,
        Self::RepositoryFetched,
        Self::NetworkStatusChanged,
        Self::GNotification,
        Self::GShowUri,
        Self::TaskAdded,
        Self::TaskRemoved,
        Self::TaskUpdated,
        Self::ProgramStarted,
        Self::ProgramFinished,
        Self::EagleStep,
        Self::EagleFinished,
        Self::DesktopEntryCreated,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManagerLocalBottlesLoaded => "Manager.local_bottles_loaded",
            Self::ForceStopNetworking => "LoadingView.stop_networking",
            Self::RepositoryFetched => "RepositoryManager.repo_fetched",
            Self::NetworkStatusChanged => "ConnectionUtils.status_changed",
            Self::GNotification => "G.send_notification",
            Self::GShowUri => "G.show_uri",
            Self::TaskAdded => "task.added",
            Self::TaskRemoved => "task.removed",
            Self::TaskUpdated => "task.updated",
            Self::ProgramStarted => "Playtime.program_started",
            Self::ProgramFinished => "Playtime.program_finished",
            Self::EagleStep => "Eagle.step",
            Self::EagleFinished => "Eagle.finished",
            Self::DesktopEntryCreated => "DesktopEntry.created",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalPayload {
    Empty,
    Boolean(bool),
    Number(i64),
    Text(String),
    Task(Uuid),
    Status(TaskStatus),
    Notification(Notification),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalEvent {
    pub signal: Signal,
    pub payload: SignalPayload,
    pub timestamp_millis: u64,
}

#[derive(Debug, Default)]
struct SignalBus {
    events: VecDeque<SignalEvent>,
    latest: HashMap<Signal, SignalPayload>,
}

impl SignalBus {
    const MAX_EVENTS: usize = 1024;

    fn send(&mut self, signal: Signal, payload: SignalPayload) {
        let timestamp_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let event = SignalEvent {
            signal,
            payload: payload.clone(),
            timestamp_millis,
        };
        if self.events.len() >= Self::MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
        self.latest.insert(signal, payload);
    }
}

fn signal_bus() -> &'static Mutex<SignalBus> {
    static BUS: OnceLock<Mutex<SignalBus>> = OnceLock::new();
    BUS.get_or_init(|| Mutex::new(SignalBus::default()))
}

fn task_registry() -> &'static Mutex<BTreeMap<Uuid, Task>> {
    static TASKS: OnceLock<Mutex<BTreeMap<Uuid, Task>>> = OnceLock::new();
    TASKS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub struct SignalManager;

impl SignalManager {
    pub fn send(signal: Signal, payload: SignalPayload) {
        if let Ok(mut bus) = signal_bus().lock() {
            bus.send(signal, payload);
        }
    }

    pub fn latest(signal: Signal) -> Option<SignalPayload> {
        signal_bus()
            .lock()
            .ok()
            .and_then(|bus| bus.latest.get(&signal).cloned())
    }

    pub fn drain() -> Vec<SignalEvent> {
        signal_bus()
            .lock()
            .map(|mut bus| bus.events.drain(..).collect())
            .unwrap_or_default()
    }

    pub fn events() -> Vec<SignalEvent> {
        signal_bus()
            .lock()
            .map(|bus| bus.events.iter().cloned().collect())
            .unwrap_or_default()
    }
}

pub struct TaskManager;

impl TaskManager {
    pub fn add(title: &str, subtitle: &str, hidden: bool, cancellable: bool) -> Uuid {
        let id = Uuid::new_v4();
        let task = Task {
            id,
            title: title.to_owned(),
            subtitle: subtitle.to_owned(),
            hidden,
            cancellable,
            status: TaskStatus::Running,
            received: 0,
            total: 0,
        };
        if let Ok(mut tasks) = task_registry().lock() {
            tasks.insert(id, task);
        }
        SignalManager::send(Signal::TaskAdded, SignalPayload::Task(id));
        id
    }

    pub fn get(id: Uuid) -> Option<Task> {
        task_registry().lock().ok()?.get(&id).cloned()
    }

    pub fn list() -> Vec<Task> {
        task_registry()
            .lock()
            .map(|tasks| tasks.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn update_subtitle(id: Uuid, subtitle: &str) {
        if let Ok(mut tasks) = task_registry().lock() {
            if let Some(task) = tasks.get_mut(&id) {
                task.subtitle = subtitle.to_owned();
            }
        }
        SignalManager::send(Signal::TaskUpdated, SignalPayload::Task(id));
    }

    pub fn stream_update(id: Uuid, received: u64, total: u64, status: TaskStatus) {
        match status {
            TaskStatus::Done | TaskStatus::Failed | TaskStatus::Cancelled => {
                Self::remove(id);
                return;
            }
            TaskStatus::Running => {}
        }
        let subtitle = if received == 0 && total == 0 {
            "Calculating…".to_owned()
        } else if total == 0 {
            String::new()
        } else {
            format!("{}%", received * 100 / total)
        };
        if let Ok(mut tasks) = task_registry().lock() {
            if let Some(task) = tasks.get_mut(&id) {
                task.subtitle = subtitle.clone();
                task.received = received;
                task.total = total;
            }
        }
        SignalManager::send(Signal::TaskUpdated, SignalPayload::Task(id));
        Self::update_subtitle(id, &subtitle);
    }

    pub fn request_cancel(id: Uuid) -> bool {
        let cancellable = Self::get(id).map(|task| task.cancellable).unwrap_or(false);
        if !cancellable {
            return false;
        }
        Self::update_subtitle(id, "Cancelling...");
        if let Ok(mut tasks) = task_registry().lock() {
            if let Some(task) = tasks.get_mut(&id) {
                task.status = TaskStatus::Cancelled;
            }
        }
        true
    }

    pub fn remove(id: Uuid) {
        if let Ok(mut tasks) = task_registry().lock() {
            tasks.remove(&id);
        }
        SignalManager::send(Signal::TaskRemoved, SignalPayload::Task(id));
    }

    #[cfg(test)]
    pub fn clear() {
        if let Ok(mut tasks) = task_registry().lock() {
            tasks.clear();
        }
        if let Ok(mut bus) = signal_bus().lock() {
            bus.events.clear();
            bus.latest.clear();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppEvent {
    ComponentsFetching,
    DependenciesFetching,
    InstallersFetching,
    ComponentsOrganizing,
    DependenciesOrganizing,
    InstallersOrganizing,
    ComponentsInstall,
}

#[derive(Debug, Default)]
struct EventState {
    set: HashSet<AppEvent>,
}

fn event_state() -> &'static (Mutex<EventState>, Condvar) {
    static STATE: OnceLock<(Mutex<EventState>, Condvar)> = OnceLock::new();
    STATE.get_or_init(|| (Mutex::new(EventState::default()), Condvar::new()))
}

pub struct EventManager;

impl EventManager {
    pub fn wait(event: AppEvent) {
        let (lock, condvar) = event_state();
        let mut state = match lock.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        while !state.set.contains(&event) {
            state = match condvar.wait(state) {
                Ok(state) => state,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
    }

    pub fn done(event: AppEvent) {
        let (lock, condvar) = event_state();
        if let Ok(mut state) = lock.lock() {
            state.set.insert(event);
        }
        condvar.notify_all();
    }

    pub fn reset(event: AppEvent) {
        let (lock, _) = event_state();
        if let Ok(mut state) = lock.lock() {
            state.set.remove(&event);
        }
    }

    pub fn is_done(event: AppEvent) -> bool {
        let (lock, _) = event_state();
        lock.lock()
            .map(|state| state.set.contains(&event))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_lifecycle_emits_signals() {
        TaskManager::clear();
        let id = TaskManager::add("Download", "", false, true);
        assert_eq!(TaskManager::get(id).unwrap().status, TaskStatus::Running);
        TaskManager::stream_update(id, 25, 100, TaskStatus::Running);
        assert_eq!(TaskManager::get(id).unwrap().subtitle, "25%");
        assert!(TaskManager::request_cancel(id));
        TaskManager::remove(id);
        assert!(TaskManager::get(id).is_none());
        assert!(SignalManager::events()
            .iter()
            .any(|event| event.signal == Signal::TaskAdded));
    }

    #[test]
    fn event_manager_done_is_immediate() {
        let event = AppEvent::InstallersOrganizing;
        EventManager::reset(event);
        assert!(!EventManager::is_done(event));
        EventManager::done(event);
        EventManager::wait(event);
        assert!(EventManager::is_done(event));
    }
}
