//! Windows notification-area shell.
//!
//! This is the only Windows-specific module: it owns the hidden message
//! window, the tray icon, the one-second timer, the context menu, and process
//! launching. All decisions live in [`crate::app`], so this file stays a thin,
//! auditable Win32 adapter. Nothing here reads Copilot state directly.

use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::{OsStr, c_void},
    fmt,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr,
};

use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT,
        WPARAM,
    },
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS,
        DeleteObject,
    },
    System::{
        LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount64, Threading::CreateMutexW,
    },
    UI::{
        Shell::{
            NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_WARNING, NIM_ADD, NIM_DELETE,
            NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CW_USEDEFAULT, CreateIconIndirect, CreatePopupMenu, CreateWindowExW,
            DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW,
            GetCursorPos, GetMessageW, HICON, ICONINFO, IDC_ARROW, KillTimer, LoadCursorW,
            MB_ICONERROR, MB_OK, MF_BYCOMMAND, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG,
            MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassExW, RegisterWindowMessageW,
            SW_SHOWNORMAL, SetForegroundWindow, SetTimer, TPM_BOTTOMALIGN, TPM_NONOTIFY,
            TPM_RETURNCMD, TPM_RIGHTALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
            WM_APP, WM_DESTROY, WM_LBUTTONDBLCLK, WM_NULL, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW,
            WS_OVERLAPPED,
        },
    },
};

use crate::{
    APPLICATION_NAME,
    actions::{LaunchTarget, MENU_ITEMS, TrayAction, default_settings_path, ensure_settings_file},
    app::{
        App, AppEffect, AppMessage, AttentionRuntime, Balloon, TrayView, diagnostic_text,
        menu_text, safe_windows_path, session_identity, session_rows, session_text, source_text,
        timer_interval_ms, write_wide_buffer,
    },
    attention::SessionIdentity,
    attention_worker::AttentionWorkers,
    config::{MonitorConfig, Provider},
    icon::IconVariant,
};

const WINDOW_CLASS_NAME: &str = "TrayGoblinShellWindow";
const TRAY_ICON_ID: u32 = 1;
const TRAY_CALLBACK_MESSAGE: u32 = WM_APP + 1;
const POLL_TIMER_ID: usize = 1;

/// Startup failures. Deliberately coarse and content-free: they name the
/// Win32 resource that could not be created, never a path or session detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayError {
    NoSessionRoot,
    WindowClassUnavailable,
    WindowUnavailable,
    IconUnavailable,
    ConfigurationUnreadable,
    ConfigurationInvalid,
    TimerUnavailable,
    WorkerUnavailable,
    SingletonUnavailable,
}

impl fmt::Display for TrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSessionRoot => write!(
                f,
                "no Copilot session folder is configured and none could be determined"
            ),
            Self::WindowClassUnavailable => {
                write!(f, "the tray window class could not be registered")
            }
            Self::WindowUnavailable => write!(f, "the tray message window could not be created"),
            Self::IconUnavailable => write!(f, "a tray icon could not be created"),
            Self::ConfigurationUnreadable => {
                write!(f, "the settings file could not be read")
            }
            Self::ConfigurationInvalid => write!(
                f,
                "the settings file is invalid; check sources and pollIntervalMs (500-10000)"
            ),
            Self::TimerUnavailable => write!(f, "the polling timer could not be started"),
            Self::WorkerUnavailable => write!(f, "attention source workers could not be started"),
            Self::SingletonUnavailable => {
                write!(f, "the interactive-session tray guard could not be created")
            }
        }
    }
}

impl std::error::Error for TrayError {}

struct Icons {
    idle: HICON,
    working: HICON,
    attention: HICON,
}

impl Icons {
    fn load() -> Result<Self, TrayError> {
        Ok(Self {
            idle: create_icon(IconVariant::Idle)?,
            working: create_icon(IconVariant::Working)?,
            attention: create_icon(IconVariant::AttentionNeeded)?,
        })
    }

    fn handle(&self, variant: IconVariant) -> HICON {
        match variant {
            IconVariant::Idle => self.idle,
            IconVariant::Working => self.working,
            IconVariant::AttentionNeeded => self.attention,
        }
    }
}

struct Shell {
    window: HWND,
    app: App,
    attention: AttentionRuntime,
    icons: Icons,
    taskbar_created_message: u32,
    icon_registered: bool,
}

thread_local! {
    static SHELL: RefCell<Option<Shell>> = const { RefCell::new(None) };
}

/// Runs the tray until the user selects Quit.
pub fn run() -> Result<(), TrayError> {
    // A Local named object is scoped to this interactive Windows session, not
    // a service/global collector. Keep the handle alive until the loop exits.
    let Some(_singleton) = Singleton::acquire()? else {
        return Ok(());
    };
    let settings_path = default_settings_path();
    let config = load_config(settings_path.as_deref())?;
    let sources = config
        .resolved_sources()
        .map_err(|_| TrayError::ConfigurationInvalid)?;
    let session_root = sources
        .iter()
        .find(|source| source.kind == Provider::Copilot)
        .map(|source| source.path.clone());
    let workers = AttentionWorkers::new(sources, unix_epoch_ms())
        .map_err(|_| TrayError::WorkerUnavailable)?;
    let timer_interval = timer_interval_ms(config.poll_interval_ms());

    let window = unsafe { create_message_window()? };
    let icons = match Icons::load() {
        Ok(icons) => icons,
        Err(error) => {
            unsafe { DestroyWindow(window) };
            return Err(error);
        }
    };
    let app = App::new(&config, session_root, settings_path);
    let attention = AttentionRuntime::new(workers);

    let taskbar_created_message =
        unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) };

    let mut shell = Shell {
        window,
        app,
        attention,
        icons,
        taskbar_created_message,
        icon_registered: false,
    };
    // Explorer may still be starting when a Startup shortcut launches us.
    // Initial notification-area failure is retryable on every timer tick and
    // on the registered TaskbarCreated broadcast.
    shell.readd_icon();

    let now = unsafe { GetTickCount64() };
    shell.dispatch(AppMessage::Started { now_ms: now });

    unsafe {
        if SetTimer(window, POLL_TIMER_ID, timer_interval, None) == 0 {
            DestroyWindow(window);
            return Err(TrayError::TimerUnavailable);
        }
        SHELL.with(|cell| *cell.borrow_mut() = Some(shell));
        run_message_loop();
        KillTimer(window, POLL_TIMER_ID);
        DestroyWindow(window);
    }

    Ok(())
}

/// Reads the user's JSON settings when present. Missing settings use the
/// documented defaults; unreadable or invalid settings fail explicitly with
/// a content-free startup error.
fn load_config(settings_path: Option<&Path>) -> Result<MonitorConfig, TrayError> {
    let Some(path) = settings_path else {
        return Ok(MonitorConfig::default());
    };

    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MonitorConfig::default());
        }
        Err(_) => return Err(TrayError::ConfigurationUnreadable),
    };

    MonitorConfig::parse(&contents).map_err(|_| TrayError::ConfigurationInvalid)
}

/// Release builds use the Windows GUI subsystem, so stderr is not visible.
/// Surface startup failures through a native, content-free dialog.
pub fn show_startup_error(error: &TrayError) {
    let title = wide("TrayGoblin could not start");
    let message = wide(&error.to_string());
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

unsafe fn create_message_window() -> Result<HWND, TrayError> {
    let class_name = wide(WINDOW_CLASS_NAME);
    let window_name = wide(APPLICATION_NAME);
    let instance = unsafe { GetModuleHandleW(ptr::null()) };

    let mut class: WNDCLASSEXW = unsafe { std::mem::zeroed() };
    class.cbSize = size_of::<WNDCLASSEXW>() as u32;
    class.lpfnWndProc = Some(window_proc);
    class.hInstance = instance;
    class.hCursor = unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) };
    class.lpszClassName = class_name.as_ptr();

    if unsafe { RegisterClassExW(&class) } == 0 {
        return Err(TrayError::WindowClassUnavailable);
    }

    // A hidden, never-shown window: the notification area needs an HWND to
    // deliver mouse callbacks, but TrayGoblin has no visible window.
    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            ptr::null(),
        )
    };

    if window.is_null() {
        return Err(TrayError::WindowUnavailable);
    }

    Ok(window)
}

unsafe fn run_message_loop() {
    let mut message: MSG = unsafe { std::mem::zeroed() };
    loop {
        let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
        if result <= 0 {
            return;
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

impl Shell {
    /// Runs the application state machine until it stops producing work.
    /// Effects are queued rather than recursed so a poll that triggers a tray
    /// update cannot grow the stack.
    fn dispatch(&mut self, message: AppMessage) {
        let refresh = matches!(
            message,
            AppMessage::MenuCommand {
                action: TrayAction::RefreshNow,
                ..
            }
        );
        let drive = refresh
            || matches!(
                message,
                AppMessage::Started { .. } | AppMessage::TimerTick { .. }
            );
        let mut poll = false;
        let mut queue = VecDeque::from([message]);

        while let Some(message) = queue.pop_front() {
            for effect in self.app.handle(message) {
                match effect {
                    AppEffect::RequestPoll => poll = true,
                    AppEffect::UpdateTray(view) => self.update_icon(&view),
                    AppEffect::Launch { action, target } => {
                        if launch(&target).is_err() {
                            queue.push_back(AppMessage::LaunchFailed { action });
                        }
                    }
                    AppEffect::Quit => {
                        self.attention.stop();
                        self.remove_icon();
                        unsafe { PostQuitMessage(0) };
                    }
                }
            }
        }
        if drive {
            let window = self.window;
            let registered = self.icon_registered;
            let mut failed = false;
            self.attention
                .cycle(poll, refresh, unix_epoch_ms, |balloon| {
                    let submitted = registered && submit_balloon(window, balloon);
                    failed = !submitted;
                    submitted
                });
            // Icon recovery never replaces/rearms the worker/controller engine.
            if failed {
                self.icon_registered = false;
            }
            for effect in self.app.update_attention(
                self.attention.snapshot().clone(),
                self.attention.submission_failed(),
            ) {
                if let AppEffect::UpdateTray(view) = effect {
                    self.update_icon(&view);
                }
            }
        }
    }

    fn notify_icon_data(&self, view: Option<&TrayView>) -> NOTIFYICONDATAW {
        let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = self.window;
        data.uID = TRAY_ICON_ID;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = TRAY_CALLBACK_MESSAGE;

        let view = view.cloned().unwrap_or_else(|| self.app.render_view());
        data.hIcon = self.icons.handle(view.variant);
        write_wide_buffer(&mut data.szTip, &view.tooltip);
        data
    }

    fn update_icon(&mut self, view: &TrayView) {
        let data = self.notify_icon_data(Some(view));
        let message = if self.icon_registered {
            NIM_MODIFY
        } else {
            NIM_ADD
        };
        let updated = unsafe { Shell_NotifyIconW(message, &data) } != 0;
        // A failed balloon/update does not prove whether Explorer retained the
        // icon. Recover either case without touching delivery history.
        let fallback = if message == NIM_ADD {
            NIM_MODIFY
        } else {
            NIM_ADD
        };
        self.icon_registered = updated || unsafe { Shell_NotifyIconW(fallback, &data) } != 0;
    }

    /// Explorer restarts drop every tray icon; re-adding on `TaskbarCreated`
    /// keeps TrayGoblin visible without restarting the process.
    fn readd_icon(&mut self) {
        self.icon_registered = false;
        let view = self.app.render_view();
        self.update_icon(&view);
    }

    fn remove_icon(&mut self) {
        if !self.icon_registered {
            return;
        }
        let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = self.window;
        data.uID = TRAY_ICON_ID;
        unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
        self.icon_registered = false;
    }

    /// Shows the right-click menu. `TPM_RETURNCMD` keeps the selection
    /// synchronous, so no `WM_COMMAND` can re-enter the window procedure
    /// while the shell state is checked out.
    fn show_menu(&mut self) -> Option<MenuSelection> {
        let menu = unsafe { CreatePopupMenu() };
        if menu.is_null() {
            return None;
        }

        let mut sessions = Vec::new();
        if let Some(snapshot) = self.app.attention_snapshot() {
            let view = self.app.render_view();
            append_menu_text(menu, MF_GRAYED, 0, &view.tooltip);
            append_menu_text(
                menu,
                MF_GRAYED,
                0,
                "Counts: A attention | W working | I idle | U unavailable | D diagnostics",
            );
            if self.attention.submission_failed() {
                append_menu_text(
                    menu,
                    MF_GRAYED,
                    0,
                    "Submission failed; attention retained. Refresh can rearm exhausted attempts.",
                );
            }
            unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null()) };
            for (source, session) in session_rows(snapshot) {
                let submenu = unsafe { CreatePopupMenu() };
                if submenu.is_null() {
                    continue;
                }
                let id = FIRST_SESSION_COMMAND_ID + sessions.len() as u32;
                append_menu_text(
                    submenu,
                    MF_STRING,
                    id as usize,
                    "Use for Open in VS Code (no terminal focus)",
                );
                for detail in [
                    format!("Source: {} ({})", source.source.label, source.source.id),
                    format!("Session: {}", session.session_id),
                    format!("Instance: {}", session.instance_id),
                    format!("Label: {}", session.label.as_deref().unwrap_or("unlabeled")),
                    format!("Model: {}", session.model.as_deref().unwrap_or("unknown")),
                    format!(
                        "Directory: {}",
                        session.directory.as_deref().unwrap_or("unknown")
                    ),
                    format!(
                        "Run start (UNIX ms): {} | Last run (ms): {}",
                        session
                            .run_started_at_ms
                            .map_or("unknown".into(), |n| n.to_string()),
                        session
                            .last_run_duration_ms
                            .map_or("unknown".into(), |n| n.to_string())
                    ),
                ] {
                    append_menu_text(submenu, MF_GRAYED, 0, &detail);
                }
                append_menu_text(
                    menu,
                    MF_POPUP,
                    submenu as usize,
                    &session_text(source, session),
                );
                sessions.push(session_identity(source, session));
            }
            if sessions.is_empty() {
                append_menu_text(menu, MF_GRAYED, 0, "No live sessions");
            }
            unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null()) };
            for source in &snapshot.sources {
                let submenu = unsafe { CreatePopupMenu() };
                if submenu.is_null() {
                    continue;
                }
                append_menu_text(submenu, MF_GRAYED, 0, &source_text(source));
                append_menu_text(
                    submenu,
                    MF_GRAYED,
                    0,
                    &format!("Unavailable pi files: {}", source.unavailable_pi_statuses),
                );
                if let Some(distro) = &source.source.wsl_distro {
                    append_menu_text(
                        submenu,
                        MF_GRAYED,
                        0,
                        &format!("WSL: {distro}; UNC reads may wake distro"),
                    );
                }
                for diagnostic in &source.diagnostics {
                    append_menu_text(submenu, MF_GRAYED, 0, diagnostic_text(*diagnostic));
                }
                append_menu_text(menu, MF_POPUP, submenu as usize, &source_text(source));
            }
            unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null()) };
        }

        for action in MENU_ITEMS {
            if action == TrayAction::Quit {
                unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null()) };
            }
            let label = wide(action.label());
            unsafe {
                AppendMenuW(
                    menu,
                    MF_STRING | MF_BYCOMMAND,
                    action.command_id() as usize,
                    label.as_ptr(),
                )
            };
        }

        let mut cursor = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut cursor) };
        // Required so the menu closes when the user clicks elsewhere.
        unsafe { SetForegroundWindow(self.window) };

        let selection = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY,
                cursor.x,
                cursor.y,
                0,
                self.window,
                ptr::null(),
            )
        };
        // Documented Win32 workaround: without a follow-up message the menu
        // can stay on screen after the user clicks elsewhere.
        unsafe {
            PostMessageW(self.window, WM_NULL, 0, 0);
            DestroyMenu(menu);
        };

        let selection = selection as u32;
        TrayAction::from_command_id(selection)
            .map(MenuSelection::Action)
            .or_else(|| {
                selection
                    .checked_sub(FIRST_SESSION_COMMAND_ID)
                    .and_then(|i| sessions.get(i as usize).cloned())
                    .map(MenuSelection::Session)
            })
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.attention.stop();
        self.remove_icon();
        for icon in [self.icons.idle, self.icons.working, self.icons.attention] {
            unsafe { DestroyIcon(icon) };
        }
    }
}

/// Borrows the shell state without holding a `RefCell` guard across Win32
/// calls, so a re-entrant message cannot panic on a double borrow.
fn with_shell<R>(action: impl FnOnce(&mut Shell) -> R) -> Option<R> {
    let mut shell = SHELL.with(|cell| cell.borrow_mut().take())?;
    let result = action(&mut shell);
    SHELL.with(|cell| *cell.borrow_mut() = Some(shell));
    Some(result)
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TIMER => {
            let now_ms = unsafe { GetTickCount64() };
            with_shell(|shell| {
                if !shell.icon_registered {
                    shell.readd_icon();
                }
                shell.dispatch(AppMessage::TimerTick { now_ms });
            });
            0
        }
        TRAY_CALLBACK_MESSAGE => {
            handle_tray_callback(lparam as u32);
            0
        }
        WM_DESTROY => {
            let message_loop_started = SHELL.with(|cell| cell.borrow_mut().take().is_some());
            if message_loop_started {
                unsafe { PostQuitMessage(0) };
            }
            0
        }
        _ => {
            let taskbar_created =
                with_shell(|shell| shell.taskbar_created_message).unwrap_or_default();
            if taskbar_created != 0 && message == taskbar_created {
                with_shell(|shell| shell.readd_icon());
                return 0;
            }
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
    }
}

fn handle_tray_callback(mouse_message: u32) {
    match mouse_message {
        WM_RBUTTONUP => {
            let selection = with_shell(|shell| shell.show_menu()).flatten();
            match selection {
                Some(MenuSelection::Action(action)) => {
                    let now_ms = unsafe { GetTickCount64() };
                    with_shell(|shell| shell.dispatch(AppMessage::MenuCommand { action, now_ms }));
                }
                Some(MenuSelection::Session(identity)) => {
                    with_shell(|shell| shell.app.select_session(identity));
                }
                None => {}
            }
        }
        // Double-clicking is the fastest path to the primary action.
        WM_LBUTTONDBLCLK => {
            let now_ms = unsafe { GetTickCount64() };
            with_shell(|shell| {
                shell.dispatch(AppMessage::MenuCommand {
                    action: TrayAction::RefreshNow,
                    now_ms,
                })
            });
        }
        _ => {}
    }
}

/// Opens a local target. Only the target kind reaches the operating system;
/// failures are reported back as a content-free action error.
fn launch(target: &LaunchTarget) -> Result<(), ()> {
    // Never let POSIX/URI/quoted paths acquire accidental Windows meanings.
    let path = match target {
        LaunchTarget::Editor { directory } | LaunchTarget::Folder { directory } => directory,
        LaunchTarget::File { path } => path,
    };
    if !safe_windows_path(&path.to_string_lossy()) {
        return Err(());
    }
    match target {
        LaunchTarget::Editor { directory } => {
            let quoted = format!("\"{}\"", directory.display());
            // Try the standard VS Code launchers directly. A blind `cmd /C`
            // fallback would only prove cmd.exe started, hiding a missing
            // `code` command and making LaunchFailed unreachable.
            for program in ["code.cmd", "code.exe", "code"] {
                if shell_execute(program, Some(&quoted), Some(directory), SW_SHOWNORMAL).is_ok() {
                    return Ok(());
                }
            }
            Err(())
        }
        LaunchTarget::Folder { directory } => {
            shell_execute(&directory.display().to_string(), None, None, SW_SHOWNORMAL)
        }
        LaunchTarget::File { path } => {
            ensure_settings_file(path).map_err(|_| ())?;
            let file = path.display().to_string();
            shell_execute(&file, None, None, SW_SHOWNORMAL).or_else(|()| {
                // A machine without a JSON handler still gets an editor.
                shell_execute(
                    "notepad.exe",
                    Some(&format!("\"{file}\"")),
                    None,
                    SW_SHOWNORMAL,
                )
            })
        }
    }
}

fn shell_execute(
    file: &str,
    parameters: Option<&str>,
    directory: Option<&Path>,
    show: i32,
) -> Result<(), ()> {
    let file = wide(file);
    let operation = wide("open");
    let parameters = parameters.map(wide);
    let directory = directory.map(|path| wide(&path.display().to_string()));

    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            parameters
                .as_ref()
                .map_or(ptr::null(), |value| value.as_ptr()),
            directory
                .as_ref()
                .map_or(ptr::null(), |value| value.as_ptr()),
            show,
        )
    };

    // ShellExecuteW returns a value greater than 32 on success.
    if result as usize > 32 {
        Ok(())
    } else {
        Err(())
    }
}

/// Builds a 32-bit ARGB icon from the platform-neutral glyph renderer.
fn create_icon(variant: IconVariant) -> Result<HICON, TrayError> {
    let image = variant.render().map_err(|_| TrayError::IconUnavailable)?;
    let width = image.width() as i32;
    let height = image.height() as i32;

    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        // Negative height selects a top-down bitmap, matching the renderer.
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let mut pixels: *mut c_void = ptr::null_mut();
    let color_bitmap = unsafe {
        CreateDIBSection(
            ptr::null_mut(),
            &info,
            DIB_RGB_COLORS,
            &mut pixels,
            ptr::null_mut(),
            0,
        )
    };
    if color_bitmap.is_null() || pixels.is_null() {
        if !color_bitmap.is_null() {
            unsafe { DeleteObject(color_bitmap) };
        }
        return Err(TrayError::IconUnavailable);
    }

    let bgra = image.bgra();
    unsafe { ptr::copy_nonoverlapping(bgra.as_ptr(), pixels.cast::<u8>(), bgra.len()) };

    // A zeroed mask keeps every pixel visible; transparency comes from the
    // alpha channel of the color bitmap.
    let mask_stride = (width as usize).div_ceil(16) * 2;
    let mask_bits = vec![0u8; mask_stride * height as usize];
    let mask_bitmap =
        unsafe { CreateBitmap(width, height, 1, 1, mask_bits.as_ptr().cast::<c_void>()) };
    if mask_bitmap.is_null() {
        unsafe { DeleteObject(color_bitmap) };
        return Err(TrayError::IconUnavailable);
    }

    let icon_info = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: mask_bitmap,
        hbmColor: color_bitmap,
    };
    let icon = unsafe { CreateIconIndirect(&icon_info) };

    unsafe {
        DeleteObject(color_bitmap);
        DeleteObject(mask_bitmap);
    }

    if icon.is_null() {
        return Err(TrayError::IconUnavailable);
    }

    Ok(icon)
}

/// Shell acceptance is submission only; Focus Assist/settings can suppress it.
/// The exact text/buffer construction is shared with host policy tests.
fn submit_balloon(window: HWND, balloon: &Balloon) -> bool {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = window;
    data.uID = TRAY_ICON_ID;
    data.uFlags = NIF_INFO;
    data.dwInfoFlags = NIIF_WARNING;
    write_wide_buffer(&mut data.szInfoTitle, &balloon.title);
    write_wide_buffer(&mut data.szInfo, &balloon.body);
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) != 0 }
}

/// Separate from GetTickCount64: freshness/retry timestamps use wall time.
fn unix_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

const FIRST_SESSION_COMMAND_ID: u32 = 100;
enum MenuSelection {
    Action(TrayAction),
    Session(SessionIdentity),
}

fn append_menu_text(
    menu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
    flags: u32,
    id: usize,
    text: &str,
) {
    let text = wide(&menu_text(text));
    unsafe { AppendMenuW(menu, flags, id, text.as_ptr()) };
}

struct Singleton(HANDLE);
impl Singleton {
    fn acquire() -> Result<Option<Self>, TrayError> {
        let name = wide("Local\\TrayGoblin.NativeTray");
        let handle = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(TrayError::SingletonUnavailable);
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return Ok(None);
        }
        Ok(Some(Self(handle)))
    }
}
impl Drop for Singleton {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
