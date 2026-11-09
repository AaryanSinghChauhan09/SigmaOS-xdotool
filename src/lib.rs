// SPDX-License-Identifier: MIT
// SigmaOS-xdotool - X11 Automation Tool component
// xdotool-style X11 window/keyboard/mouse automation for scripts,
// inspired by Arch's xdotool package.

use std::fmt;
use std::path::PathBuf;

// ============================================================================
// WINDOW INFO
// ============================================================================

/// Window geometry
#[derive(Debug, Clone, Copy, Default)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Window state flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Active,
    Iconic,
    Withdrawn,
    Normal,
}

/// A managed X11 window
#[derive(Debug, Clone)]
pub struct Window {
    /// X11 window ID
    pub id: u32,
    /// Window name/title
    pub name: String,
    /// Window class (WM_CLASS)
    pub class: String,
    /// Window class instance
    pub instance: String,
    /// Geometry
    pub geometry: Geometry,
    /// Desktop number
    pub desktop: u32,
    /// Whether window is active
    pub active: bool,
    /// Whether window is minimized
    pub minimized: bool,
    /// PID of owning process
    pub pid: u32,
}

impl Window {
    /// Whether window matches a search pattern
    pub fn matches(&self, pattern: &str) -> bool {
        self.name.contains(pattern)
            || self.class.contains(pattern)
            || self.instance.contains(pattern)
            || format!("0x{:x}", self.id).contains(pattern)
            || self.id.to_string() == pattern
    }

    /// Center coordinates
    pub fn center(&self) -> (i32, i32) {
        (self.geometry.x + self.geometry.width as i32 / 2,
         self.geometry.y + self.geometry.height as i32 / 2)
    }
}

// ============================================================================
// KEY/MOUSE EVENTS
// ============================================================================

/// Key event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    KeyPress,
    KeyRelease,
    KeyRepeat,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Button4,
    Button5,
}

impl MouseButton {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Middle => "middle",
            Self::Right => "right",
            Self::WheelUp => "wheelup",
            Self::WheelDown => "wheeldown",
            Self::WheelLeft => "wheelleft",
            Self::WheelRight => "wheelright",
            Self::Button4 => "button4",
            Self::Button5 => "button5",
        }
    }
}

/// A key/mouse event to send
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key { keysym: String, event: KeyEvent },
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton, repeat: u32 },
    MouseDown { button: MouseButton },
    MouseUp { button: MouseButton },
    TypeText { text: String },
    KeyChord { keys: Vec<String> }, // e.g. ["ctrl", "c"]
}

// ============================================================================
// XDOTOOL COMMANDS
// ============================================================================

/// xdotool command types
#[derive(Debug, Clone)]
pub enum XdoCommand {
    /// Search for windows
    Search { pattern: String, class_only: bool },
    /// Get window geometry
    GetGeometry { window: String },
    /// Get window name
    GetWindowName { window: String },
    /// Get active window
    GetActiveWindow,
    /// Activate window
    Activate { window: String },
    /// Close window
    Close { window: String },
    /// Move window
    MoveWindow { window: String, x: i32, y: i32 },
    /// Resize window
    ResizeWindow { window: String, width: u32, height: u32 },
    /// Move+resize window
    WindowMoveResize { window: String, x: i32, y: i32, width: u32, height: u32 },
    /// Send keystrokes
    Key { keysym: String },
    /// Type text
    Type { text: String },
    /// Mouse move
    MouseMove { x: i32, y: i32 },
    /// Mouse click
    Click { button: MouseButton, repeat: u32 },
    /// Get mouse position
    GetMousePosition,
    /// List windows
    ListWindows,
    /// Get desktop info
    GetDesktop,
    /// Set desktop
    SetDesktop { num: u32 },
    /// Get display info
    GetDisplay,
}

/// Build xdotool command args
pub fn build_args(cmd: &XdoCommand) -> Vec<String> {
    let mut args = vec!["xdotool".to_string()];
    match cmd {
        XdoCommand::Search { pattern, class_only } => {
            args.push("search".to_string());
            if *class_only { args.push("--class".to_string()); }
            args.push(pattern.clone());
        }
        XdoCommand::GetGeometry { window } => {
            args.push("getwindowgeometry".to_string());
            args.push(window.clone());
        }
        XdoCommand::GetWindowName { window } => {
            args.push("getwindowname".to_string());
            args.push(window.clone());
        }
        XdoCommand::GetActiveWindow => {
            args.push("getactivewindow".to_string());
        }
        XdoCommand::Activate { window } => {
            args.push("windowactivate".to_string());
            args.push(window.clone());
        }
        XdoCommand::Close { window } => {
            args.push("windowclose".to_string());
            args.push(window.clone());
        }
        XdoCommand::MoveWindow { window, x, y } => {
            args.push("windowmove".to_string());
            args.push(window.clone());
            args.push(x.to_string());
            args.push(y.to_string());
        }
        XdoCommand::ResizeWindow { window, width, height } => {
            args.push("windowsize".to_string());
            args.push(window.clone());
            args.push(width.to_string());
            args.push(height.to_string());
        }
        XdoCommand::WindowMoveResize { window, x, y, width, height } => {
            args.push("windowsize".to_string());
            args.push(window.clone());
            args.push(format!("{}x{}", width, height));
            args.push("windowmove".to_string());
            args.push(window.clone());
            args.push(x.to_string());
            args.push(y.to_string());
        }
        XdoCommand::Key { keysym } => {
            args.push("key".to_string());
            args.push(keysym.clone());
        }
        XdoCommand::Type { text } => {
            args.push("type".to_string());
            args.push(text.clone());
        }
        XdoCommand::MouseMove { x, y } => {
            args.push("mousemove".to_string());
            args.push(x.to_string());
            args.push(y.to_string());
        }
        XdoCommand::Click { button, repeat } => {
            args.push("click".to_string());
            if *repeat > 1 {
                args.push(repeat.to_string());
            }
            args.push(button.as_str().to_string());
        }
        XdoCommand::GetMousePosition => {
            args.push("getmouselocation".to_string());
        }
        XdoCommand::ListWindows => {
            args.push("search".to_string());
            args.push("--name".to_string());
            args.push(".*".to_string());
        }
        XdoCommand::GetDesktop => {
            args.push("get_desktop".to_string());
        }
        XdoCommand::SetDesktop { num } => {
            args.push("set_desktop".to_string());
            args.push(num.to_string());
        }
        XdoCommand::GetDisplay => {
            args.push("getdisplaygeometry".to_string());
        }
    }
    args
}

// ============================================================================
// WINDOW MANAGER
// ============================================================================

/// Manages X11 windows (simulated)
pub struct WindowManager {
    windows: Vec<Window>,
    active_window: Option<u32>,
    mouse_x: i32,
    mouse_y: i32,
    current_desktop: u32,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            active_window: None,
            mouse_x: 0,
            mouse_y: 0,
            current_desktop: 0,
        }
    }

    /// Add a window
    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    /// Remove a window by ID
    pub fn remove_window(&mut self, id: u32) {
        self.windows.retain(|w| w.id != id);
        if self.active_window == Some(id) {
            self.active_window = None;
        }
    }

    /// Get window by ID
    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Find windows matching pattern
    pub fn search(&self, pattern: &str) -> Vec<&Window> {
        self.windows.iter().filter(|w| w.matches(pattern)).collect()
    }

    /// Get active window
    pub fn active_window(&self) -> Option<&Window> {
        self.active_window.and_then(|id| self.get_window(id))
    }

    /// Activate a window
    pub fn activate(&mut self, window_id: u32) -> Result<(), XdoError> {
        if self.get_window(window_id).is_none() {
            return Err(XdoError::WindowNotFound(window_id));
        }
        self.active_window = Some(window_id);
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
            w.active = true;
        }
        // Deactivate others
        for w in self.windows.iter_mut() {
            if w.id != window_id {
                w.active = false;
            }
        }
        Ok(())
    }

    /// Close a window
    pub fn close_window(&mut self, window_id: u32) -> Result<(), XdoError> {
        if self.get_window(window_id).is_none() {
            return Err(XdoError::WindowNotFound(window_id));
        }
        self.remove_window(window_id);
        Ok(())
    }

    /// Move window
    pub fn move_window(&mut self, window_id: u32, x: i32, y: i32) -> Result<(), XdoError> {
        let w = self.windows.iter_mut()
            .find(|w| w.id == window_id)
            .ok_or(XdoError::WindowNotFound(window_id))?;
        w.geometry.x = x;
        w.geometry.y = y;
        Ok(())
    }

    /// Resize window
    pub fn resize_window(&mut self, window_id: u32, width: u32, height: u32) -> Result<(), XdoError> {
        let w = self.windows.iter_mut()
            .find(|w| w.id == window_id)
            .ok_or(XdoError::WindowNotFound(window_id))?;
        w.geometry.width = width;
        w.geometry.height = height;
        Ok(())
    }

    /// Minimize window
    pub fn minimize(&mut self, window_id: u32) -> Result<(), XdoError> {
        let w = self.windows.iter_mut()
            .find(|w| w.id == window_id)
            .ok_or(XdoError::WindowNotFound(window_id))?;
        w.minimized = true;
        w.active = false;
        Ok(())
    }

    /// Restore window
    pub fn restore(&mut self, window_id: u32) -> Result<(), XdoError> {
        let w = self.windows.iter_mut()
            .find(|w| w.id == window_id)
            .ok_or(XdoError::WindowNotFound(window_id))?;
        w.minimized = false;
        Ok(())
    }

    /// Move mouse
    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    /// Get mouse position
    pub fn mouse_position(&self) -> (i32, i32) {
        (self.mouse_x, self.mouse_y)
    }

    /// Get all windows
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// Get current desktop
    pub fn desktop(&self) -> u32 {
        self.current_desktop
    }

    /// Switch desktop
    pub fn set_desktop(&mut self, num: u32) {
        self.current_desktop = num;
    }

    /// List windows on current desktop
    pub fn windows_on_desktop(&self) -> Vec<&Window> {
        self.windows.iter()
            .filter(|w| w.desktop == self.current_desktop)
            .collect()
    }
}

impl Default for WindowManager {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum XdoError {
    WindowNotFound(u32),
    NoWindowFound,
    InvalidKey(String),
    Io(String),
}

impl fmt::Display for XdoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowNotFound(id) => write!(f, "window not found: 0x{:x}", id),
            Self::NoWindowFound => write!(f, "no window found matching pattern"),
            Self::InvalidKey(k) => write!(f, "invalid key: {}", k),
            Self::Io(m) => write!(f, "io error: {}", m),
        }
    }
}
impl std::error::Error for XdoError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_window(id: u32, name: &str) -> Window {
        Window {
            id,
            name: name.to_string(),
            class: "test".to_string(),
            instance: "test".to_string(),
            geometry: Geometry { x: 0, y: 0, width: 800, height: 600 },
            desktop: 0,
            active: false,
            minimized: false,
            pid: 1000,
        }
    }

    #[test]
    fn test_window_matches() {
        let w = test_window(0x1234, "Firefox");
        assert!(w.matches("Firefox"));
        assert!(!w.matches("firefox")); // case-sensitive match
        assert!(w.matches("0x1234"));
        assert!(!w.matches("Chrome"));
    }

    #[test]
    fn test_window_search() {
        let mut wm = WindowManager::new();
        wm.add_window(test_window(0x100, "Firefox"));
        wm.add_window(test_window(0x200, "Chrome"));
        wm.add_window(test_window(0x300, "Terminal"));

        let results = wm.search("Fire");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 0x100);
    }

    #[test]
    fn test_activate_window() {
        let mut wm = WindowManager::new();
        wm.add_window(test_window(0x100, "Window1"));
        wm.add_window(test_window(0x200, "Window2"));

        wm.activate(0x100).unwrap();
        assert_eq!(wm.active_window().unwrap().id, 0x100);
        assert!(wm.get_window(0x100).unwrap().active);
        assert!(!wm.get_window(0x200).unwrap().active);

        wm.activate(0x200).unwrap();
        assert_eq!(wm.active_window().unwrap().id, 0x200);
        assert!(!wm.get_window(0x100).unwrap().active);
    }

    #[test]
    fn test_move_resize() {
        let mut wm = WindowManager::new();
        wm.add_window(test_window(0x100, "Window"));

        wm.move_window(0x100, 100, 200).unwrap();
        assert_eq!(wm.get_window(0x100).unwrap().geometry.x, 100);
        assert_eq!(wm.get_window(0x100).unwrap().geometry.y, 200);

        wm.resize_window(0x100, 1024, 768).unwrap();
        assert_eq!(wm.get_window(0x100).unwrap().geometry.width, 1024);
        assert_eq!(wm.get_window(0x100).unwrap().geometry.height, 768);
    }

    #[test]
    fn test_close_window() {
        let mut wm = WindowManager::new();
        wm.add_window(test_window(0x100, "Window"));
        wm.activate(0x100).unwrap();
        wm.close_window(0x100).unwrap();
        assert!(wm.get_window(0x100).is_none());
        assert!(wm.active_window().is_none());
    }

    #[test]
    fn test_mouse_position() {
        let mut wm = WindowManager::new();
        wm.mouse_move(500, 300);
        assert_eq!(wm.mouse_position(), (500, 300));
    }

    #[test]
    fn test_desktop() {
        let mut wm = WindowManager::new();
        wm.set_desktop(2);
        assert_eq!(wm.desktop(), 2);
    }

    #[test]
    fn test_build_args() {
        let args = build_args(&XdoCommand::Activate { window: "0x1234".into() });
        assert!(args.contains(&"windowactivate".to_string()));
        assert!(args.contains(&"0x1234".to_string()));

        let args = build_args(&XdoCommand::Key { keysym: "Return".into() });
        assert!(args.contains(&"key".to_string()));
        assert!(args.contains(&"Return".to_string()));

        let args = build_args(&XdoCommand::Click { button: MouseButton::Left, repeat: 2 });
        assert!(args.contains(&"click".to_string()));
        assert!(args.contains(&"2".to_string()));
        assert!(args.contains(&"left".to_string()));
    }

    #[test]
    fn test_window_center() {
        let w = Window {
            id: 1, name: "Test".into(), class: "test".into(), instance: "test".into(),
            geometry: Geometry { x: 100, y: 50, width: 800, height: 600 },
            desktop: 0, active: false, minimized: false, pid: 1,
        };
        assert_eq!(w.center(), (500, 350));
    }
}
