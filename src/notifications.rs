//! Windows notification-center integration. The worker owns its WinRT listener;
//! it posts WM_NOTIFICATION to the UI thread when a new toast is discovered.
use std::{collections::HashSet, sync::{Mutex, OnceLock}, thread, time::Duration};
use windows::{
    UI::Notifications::{Management::{UserNotificationListener, UserNotificationListenerAccessStatus}, NotificationKinds, UserNotification},
    Win32::{Foundation::{HWND, LPARAM, WPARAM}, System::Com::{CoInitializeEx, COINIT_MULTITHREADED}, UI::WindowsAndMessaging::PostMessageW},
};

pub const WM_NOTIFICATION: u32 = 0x8000 + 12;
#[derive(Clone, Debug)]
pub struct Notification { pub app: String, pub title: String, pub body: String }
static LATEST: OnceLock<Mutex<Option<Notification>>> = OnceLock::new();
fn slot() -> &'static Mutex<Option<Notification>> { LATEST.get_or_init(|| Mutex::new(None)) }
pub fn take_latest() -> Option<Notification> { slot().lock().ok()?.take() }

pub fn start(hwnd: isize) {
    thread::Builder::new().name("nock-notifications".into()).spawn(move || {
        unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
        if let Err(e) = run(hwnd) { eprintln!("[Nock notifications] {e}"); }
    }).ok();
}

fn run(hwnd: isize) -> windows::core::Result<()> {
    let listener = UserNotificationListener::Current()?;
    let access = listener.RequestAccessAsync()?.join()?;
    if access != UserNotificationListenerAccessStatus::Allowed {
        eprintln!("[Nock notifications] Windows notification access was not granted");
        return Ok(());
    }
    let mut seen = HashSet::<u32>::new();
    // Existing notifications are intentionally ignored at startup.
    if let Ok(op) = listener.GetNotificationsAsync(NotificationKinds::Toast) {
        if let Ok(list) = op.join() {
            for i in 0..list.Size().unwrap_or(0) {
                if let Ok(n) = list.GetAt(i) { if let Ok(id) = n.Id() { seen.insert(id); } }
            }
        }
    }
    loop {
        thread::sleep(Duration::from_millis(650));
        let Ok(op) = listener.GetNotificationsAsync(NotificationKinds::Toast) else { continue };
        let Ok(list) = op.join() else { continue };
        let mut live = HashSet::new();
        for i in 0..list.Size().unwrap_or(0) {
            let Ok(n) = list.GetAt(i) else { continue };
            let Ok(id) = n.Id() else { continue };
            live.insert(id);
            if !seen.insert(id) { continue; }
            let item = parse(n);
            if item.title.is_empty() && item.body.is_empty() { continue; }
            if let Ok(mut latest) = slot().lock() { *latest = Some(item); }
            unsafe { let _ = PostMessageW(Some(HWND(hwnd as *mut _)), WM_NOTIFICATION, WPARAM(0), LPARAM(0)); }
        }
        seen.retain(|id| live.contains(id));
    }
}

fn parse(n: UserNotification) -> Notification {
    let app = n.AppInfo().ok().and_then(|a| a.AppUserModelId().ok()).map(|s| s.to_string()).unwrap_or_else(|| "Notification".into());
    let mut parts = Vec::new();
    if let Ok(toast) = n.Notification() {
        if let Ok(visual) = toast.Visual() {
            if let Ok(bindings) = visual.Bindings() {
                if let Some(binding) = bindings.into_iter().next() {
                    if let Ok(texts) = binding.GetTextElements() {
                        for text in texts { if let Ok(s) = text.Text() { let s = s.to_string(); if !s.trim().is_empty() { parts.push(s); } } }
                    }
                }
            }
        }
    }
    let title = if parts.is_empty() { String::new() } else { parts.remove(0) };
    Notification { app, title, body: parts.join(" · ") }
}
