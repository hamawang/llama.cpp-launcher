use crate::config::settings::{
    is_rpc_binary_name, is_server_binary_name, AppSettings, SettingsManager,
};
use crate::engine::rpc::{RpcManager, RpcState};
use crate::engine::server::{ServerManager, ServerState};
use crate::i18n::{self, Language};
use crate::spacing_debugger::SpacingDebugger;
use crate::ui::{
    launch_commands_panel, log_panel, model_panel, params_panel, presets_panel, rpc_panel,
    server_panel,
};

pub struct LlamaLauncherApp {
    settings: AppSettings,
    settings_manager: SettingsManager,
    server_manager: ServerManager,
    rpc_manager: RpcManager,
    tab_selected: String,
    show_about: bool,
    lang: Language,
    auto_start_server_on_first_frame: bool, // 新增
    start_minimized: bool,                  // 开机自启时最小化到任务栏
    debug_mode: bool,                       // egui Inspector / 调试模式开关
    spacing_debugger: SpacingDebugger,      // UI 间距可视化工具
}

impl LlamaLauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_minimized: bool) -> Self {
        let settings_manager = SettingsManager::new();
        let mut settings = settings_manager.load().unwrap_or_default();

        // 应用自启动预设
        if let Some(ref preset_name) = settings.auto_start_preset_name {
            if let Some(preset) = settings.presets.iter().find(|p| p.name == *preset_name) {
                preset.clone().apply_to(&mut settings);
            }
        }

        let auto_start_server_on_first_frame = settings.auto_start_preset_name.is_some();

        let locale = sys_locale::get_locale().unwrap_or_default();
        let lang = if locale.starts_with("zh") {
            Language::Zh
        } else {
            Language::En
        };

        let server_manager = ServerManager::new();
        let rpc_manager = RpcManager::new();

        // 全局 UI 放大 1.5 倍
        cc.egui_ctx.set_zoom_factor(1.5);

        // 同步日志开关状态到全局标志
        crate::set_log_to_file(settings.log_to_file);

        Self {
            settings,
            settings_manager,
            server_manager,
            rpc_manager,
            tab_selected: i18n::t(i18n::Key::TabServer, &lang).to_string(),
            show_about: false,
            lang,
            auto_start_server_on_first_frame,
            start_minimized,
            debug_mode: false,
            spacing_debugger: SpacingDebugger::new(),
        }
    }

    fn save(&mut self) {
        if let Err(e) = self.settings_manager.save(&self.settings) {
            log::error!("保存配置失败: {}", e);
        }
    }

    fn render_server_controls(&mut self, ui: &mut egui::Ui) {
        let server_state = self.server_manager.state();
        let start_fill = egui::Color32::from_rgb(40, 120, 40);
        let stop_fill = egui::Color32::from_rgb(180, 50, 50);
        match server_state {
            ServerState::Idle | ServerState::Error(_) => {
                let server_path_valid = self
                    .settings
                    .server_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(is_server_binary_name);
                let can_start =
                    server_path_valid && !self.settings.model_path.as_os_str().is_empty();
                let resp = ui.add_enabled(
                    can_start,
                    egui::Button::new(i18n::t(i18n::Key::BtnStartServer, &self.lang))
                        .fill(start_fill),
                );
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
                if resp.clicked() {
                    self.server_manager.start(&self.settings);
                }
            }
            ServerState::Running => {
                let resp = ui.add(
                    egui::Button::new(i18n::t(i18n::Key::BtnStopServer, &self.lang))
                        .fill(stop_fill),
                );
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
                if resp.clicked() {
                    self.server_manager.stop();
                }
            }
            ServerState::Starting | ServerState::Stopping => {
                let resp = ui.label(i18n::t(i18n::Key::StatusProcessing, &self.lang));
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
            }
        }
    }

    fn render_rpc_controls(&mut self, ui: &mut egui::Ui) {
        let rpc_state = self.rpc_manager.state();
        let rpc_start_fill = egui::Color32::from_rgb(40, 100, 140);
        let rpc_stop_fill = egui::Color32::from_rgb(180, 50, 50);
        match rpc_state {
            RpcState::Idle | RpcState::Error(_) => {
                let rpc_path_valid = self
                    .settings
                    .rpc_server_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(is_rpc_binary_name);
                let resp = ui.add_enabled(
                    rpc_path_valid,
                    egui::Button::new(i18n::t(i18n::Key::BtnStartRpc, &self.lang))
                        .fill(rpc_start_fill),
                );
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
                if resp.clicked() {
                    self.rpc_manager.start(&self.settings);
                }
            }
            RpcState::Running => {
                let resp = ui.add(
                    egui::Button::new(i18n::t(i18n::Key::BtnStopRpc, &self.lang))
                        .fill(rpc_stop_fill),
                );
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
                if resp.clicked() {
                    self.rpc_manager.stop();
                }
            }
            RpcState::Starting | RpcState::Stopping => {
                let resp = ui.label(i18n::t(i18n::Key::StatusProcessing, &self.lang));
                if self.debug_mode {
                    self.spacing_debugger.rects.push(resp.rect);
                }
            }
        }
    }

    fn render_web_client_button(&mut self, ui: &mut egui::Ui) {
        let server_running = matches!(self.server_manager.state(), ServerState::Running);
        let listening = self.server_manager.is_listening();
        let enabled = server_running && listening;
        let resp = ui.add_enabled(
            enabled,
            egui::Button::new(i18n::t(i18n::Key::BtnOpenWebClient, &self.lang)),
        );
        if self.debug_mode {
            self.spacing_debugger.rects.push(resp.rect);
        }
        if resp.clicked() {
            open_web_client_url(self.settings.port);
        }
    }
}

impl eframe::App for LlamaLauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 先获取上下文并执行不依赖面板借用的操作，再释放引用避免后面 show_inside(ui) 冲突
        {
            let ctx = ui.ctx();
            ctx.request_repaint_after(std::time::Duration::from_millis(500));

            // 根据调试模式开关，控制 egui Inspector（悬浮时显示内置检查器面板）
            // 开启时同时启用 hover_shows_next，方便查看控件之间的间距/位置关系
            #[cfg(debug_assertions)]
            {
                ctx.set_debug_on_hover(self.debug_mode);
                ctx.global_style_mut(|s| {
                    s.debug.hover_shows_next = self.debug_mode;
                });
            }
        }

        // 调试模式：每帧开始时清空间距记录
        if self.debug_mode {
            self.spacing_debugger.begin_frame();
        }

        // 应用启动时自动启动 Server
        if self.auto_start_server_on_first_frame {
            self.auto_start_server_on_first_frame = false;
            self.server_manager.start(&self.settings);
        }

        // 开机自启时最小化到任务栏（仅第一帧执行）
        if self.start_minimized {
            self.start_minimized = false;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        self.server_manager.poll_logs();
        self.rpc_manager.poll();

        if self.show_about {
            egui::Window::new(i18n::t(i18n::Key::AboutTitle, &self.lang))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([150.0, 0.0])
                .show(ui, |ui| {
                    ui.label(i18n::t(i18n::Key::AboutVersion, &self.lang));
                    ui.label(i18n::t(i18n::Key::AboutDescription, &self.lang));
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(i18n::t(i18n::Key::AboutCopyright, &self.lang))
                                .size(10.0),
                        );
                    });
                    // 关闭按钮居中显示
                    ui.horizontal_centered(|ui| {
                        if ui
                            .button(i18n::t(i18n::Key::BtnClose, &self.lang))
                            .clicked()
                        {
                            self.show_about = false;
                        }
                    });
                });
        }

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(i18n::t(i18n::Key::MenuFile, &self.lang), |ui| {
                    if ui
                        .button(i18n::t(i18n::Key::MenuItemSaveConfig, &self.lang))
                        .clicked()
                    {
                        self.save();
                    }
                    if ui
                        .button(i18n::t(i18n::Key::MenuItemLoadConfig, &self.lang))
                        .clicked()
                    {
                        if let Ok(s) = self.settings_manager.load() {
                            self.settings = s;
                        }
                    }
                    // 开机自启动
                    if ui
                        .checkbox(
                            &mut self.settings.auto_start,
                            i18n::t(i18n::Key::MenuItemAutoStart, &self.lang),
                        )
                        .changed()
                    {
                        if self.settings.auto_start {
                            enable_auto_start();
                        } else {
                            disable_auto_start();
                        }
                        // 保存设置到文件
                        if let Err(e) = self.settings_manager.save(&self.settings) {
                            log::error!("保存设置失败：{}", e);
                        }
                    }
                    // 创建桌面快捷方式
                    if ui
                        .button(i18n::t(i18n::Key::MenuItemCreateShortcut, &self.lang))
                        .clicked()
                    {
                        let _ = crate::shortcut::create_desktop_shortcut();
                    }
                });

                // 标签页切换
                let tabs = [
                    i18n::t(i18n::Key::TabServer, &self.lang),
                    i18n::t(i18n::Key::TabRpc, &self.lang),
                    i18n::t(i18n::Key::TabModel, &self.lang),
                    i18n::t(i18n::Key::TabParams, &self.lang),
                    i18n::t(i18n::Key::TabLog, &self.lang),
                    i18n::t(i18n::Key::TabCommands, &self.lang),
                    i18n::t(i18n::Key::TabPresets, &self.lang),
                ];
                for tab in &tabs {
                    let selected = self.tab_selected == *tab;
                    if ui.selectable_label(selected, *tab).clicked() {
                        self.tab_selected = tab.to_string();
                    }
                }

                ui.separator();

                // 控制按钮
                self.render_server_controls(ui);
                self.render_rpc_controls(ui);
                self.render_web_client_button(ui);

                ui.menu_button(i18n::t(i18n::Key::MenuHelp, &self.lang), |ui| {
                    if ui
                        .button(i18n::t(i18n::Key::MenuItemAbout, &self.lang))
                        .clicked()
                    {
                        self.show_about = true;
                    }
                    if ui
                        .button(i18n::t(i18n::Key::MenuItemRepo, &self.lang))
                        .clicked()
                    {
                        open_repo_url();
                    }
                    // 文件日志开关：选中时写入磁盘，不选中时静默丢弃
                    let mut log_to_file = self.settings.log_to_file;
                    if ui
                        .checkbox(
                            &mut log_to_file,
                            i18n::t(i18n::Key::MenuItemLogToFile, &self.lang),
                        )
                        .changed()
                    {
                        crate::set_log_to_file(log_to_file);
                        self.settings.log_to_file = log_to_file;
                        // 保存设置到文件
                        if let Err(e) = self.settings_manager.save(&self.settings) {
                            log::error!("保存设置失败：{}", e);
                        }
                    }
                    // 调试模式：开启 egui Inspector / 内置检查器面板
                    ui.checkbox(
                        &mut self.debug_mode,
                        i18n::t(i18n::Key::MenuItemDebugMode, &self.lang),
                    );
                });

                ui.separator();
                let status = self.server_manager.status_text(&self.lang);
                let color = if self.server_manager.is_running() {
                    egui::Color32::from_rgb(110, 255, 140)
                } else {
                    egui::Color32::GRAY
                };
                ui.colored_label(color, format!("[Server: {}]", status));
                let rpc_status = self.rpc_manager.status_text(&self.lang);
                let rpc_color = if self.rpc_manager.is_running() {
                    egui::Color32::from_rgb(110, 255, 140)
                } else {
                    egui::Color32::GRAY
                };
                ui.colored_label(rpc_color, format!("[RPC: {}]", rpc_status));
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab_selected.as_str() {
                    tab if tab == i18n::t(i18n::Key::TabServer, &self.lang) => {
                        server_panel::ui(ui, &mut self.settings, &self.settings_manager, &self.lang)
                    }
                    tab if tab == i18n::t(i18n::Key::TabRpc, &self.lang) => {
                        rpc_panel::ui(ui, &mut self.settings, &self.settings_manager, &self.lang)
                    }
                    tab if tab == i18n::t(i18n::Key::TabModel, &self.lang) => {
                        model_panel::ui(ui, &mut self.settings, &self.lang)
                    }
                    tab if tab == i18n::t(i18n::Key::TabParams, &self.lang) => {
                        params_panel::ui(ui, &mut self.settings, &self.lang)
                    }
                    tab if tab == i18n::t(i18n::Key::TabLog, &self.lang) => {
                        log_panel::ui(ui, &mut self.settings, &mut self.server_manager, &self.lang)
                    }
                    tab if tab == i18n::t(i18n::Key::TabCommands, &self.lang) => {
                        launch_commands_panel::ui(
                            ui,
                            &self.server_manager,
                            &self.rpc_manager,
                            &self.lang,
                        )
                    }
                    tab if tab == i18n::t(i18n::Key::TabPresets, &self.lang) => {
                        let should_start = presets_panel::ui(ui, &mut self.settings, &self.lang);
                        if should_start {
                            self.server_manager.start(&self.settings);
                        }
                    }

                    _ => { /* 空白面板 */ }
                }
            });
        });

        // 调试模式：绘制控件间距可视化
        if self.debug_mode {
            self.spacing_debugger.visualize(ui);
        }
    }
}

impl Drop for LlamaLauncherApp {
    fn drop(&mut self) {
        self.server_manager.stop();
        self.rpc_manager.stop();
        self.save();
    }
}

// Windows 开机自启动注册表操作函数（修正版：使用 /v，并记录错误）
#[cfg(target_os = "windows")]
fn enable_auto_start() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log::error!("获取当前 exe 路径失败: {}", e);
            return;
        }
    };

    // reg add 标准语法：reg add <Key> /v <ValueName> /d <Data> /f
    // 添加 --minimized 参数，使开机自启时程序启动后最小化到任务栏
    let path_str = exe_path.to_string_lossy().to_string();
    match std::process::Command::new("reg")
        .arg("add")
        .arg(r#"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run"#)
        .arg("/v")
        .arg("llama.cpp launcher")
        .arg("/d")
        .arg(format!("\"{}\" --minimized", path_str))
        .arg("/f")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("reg add 失败: {}", stderr.trim());
            } else {
                log::info!("开机自启注册表项已添加");
            }
        }
        Err(e) => {
            log::error!("执行 reg 命令出错: {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn disable_auto_start() {
    match std::process::Command::new("reg")
        .arg("delete")
        .arg(r#"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run"#)
        .arg("/v")
        .arg("llama.cpp launcher")
        .arg("/f")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("reg delete 失败: {}", stderr.trim());
            } else {
                log::info!("开机自启注册表项已移除");
            }
        }
        Err(e) => {
            log::error!("执行 reg 命令出错: {}", e);
        }
    }
}

// 非 Windows 平台的实现
#[cfg(not(target_os = "windows"))]
fn enable_auto_start() {
    use std::fs;

    // 获取 XDG autostart 目录
    let autostart_dir = dirs::config_dir()
        .map(|d| d.join("autostart"))
        .expect("无法获取 XDG config 目录");
    fs::create_dir_all(&autostart_dir).ok();

    // 获取当前可执行文件路径
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log::error!("获取当前 exe 路径失败: {}", e);
            return;
        }
    };

    // 创建 .desktop 文件内容
    // 添加 --minimized 参数，使开机自启时程序启动后最小化到任务栏
    let desktop_content = format!(
        r#"[Desktop Entry]
Type=Application
Name=LLama Launcher
Exec={} --minimized
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
"#,
        exe_path.display()
    );

    // 写入 autostart 目录
    let desktop_path = autostart_dir.join("llama-cpp-launcher.desktop");
    match fs::write(&desktop_path, &desktop_content) {
        Ok(_) => log::info!("XDG autostart 文件已创建: {}", desktop_path.display()),
        Err(e) => log::error!("创建 autostart 文件失败: {}", e),
    }
}

#[cfg(not(target_os = "windows"))]
fn disable_auto_start() {
    use std::fs;

    let autostart_file = dirs::config_dir().map(|d| d.join("autostart/llama-cpp-launcher.desktop"));

    if let Some(path) = autostart_file {
        match fs::remove_file(&path) {
            Ok(_) => log::info!("XDG autostart 文件已删除: {}", path.display()),
            Err(e) => log::error!("删除 autostart 文件失败: {}", e),
        }
    }
}

// 用 ShellExecuteW 打开 URL，无黑窗口 (Windows)
#[cfg(target_os = "windows")]
mod shell_execute {
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32", kind = "dylib")]
    extern "system" {
        fn ShellExecuteW(
            hind_window: *mut c_void,
            lp_operation: *const u16,
            lp_file: *const u16,
            lp_parameters: *const u16,
            lp_directory: *const u16,
            n_show_cmd: i32,
        ) -> isize;
    }

    const SW_SHOW_NORMAL: i32 = 1;

    pub(crate) fn open_url(url: &str) {
        let op_utf16 = OsStr::new("open")
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();

        let file_utf16 = OsStr::new(url)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();

        // 调用 ShellExecuteW，不产生控制台窗口
        let _res = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op_utf16.as_ptr(),
                file_utf16.as_ptr(),
                std::ptr::null::<u16>(),
                std::ptr::null::<u16>(),
                SW_SHOW_NORMAL,
            )
        };

        // 如需错误处理：_res as isize <= 32 表示失败；当前保持轻量不额外弹窗。
    }
}

// WebClient: 用系统默认浏览器打开 http://127.0.0.1:<port>
#[cfg(target_os = "windows")]
fn open_web_client_url(port: u16) {
    let url = format!("http://127.0.0.1:{}", port);
    shell_execute::open_url(&url);
}

#[cfg(not(target_os = "windows"))]
fn open_web_client_url(port: u16) {
    use std::process::Command;
    let url = format!("http://127.0.0.1:{}", port);
    // 简单 fallback，失败则忽略
    let _ = Command::new("xdg-open").arg(&url).spawn();
}

// GitHub 仓库页面
#[cfg(target_os = "windows")]
fn open_repo_url() {
    shell_execute::open_url("https://github.com/yihuishou/llama.cpp-launcher");
}

#[cfg(not(target_os = "windows"))]
fn open_repo_url() {
    use std::process::Command;
    let url = "https://github.com/yihuishou/llama.cpp-launcher";
    let _ = Command::new("xdg-open").arg(url).spawn();
}
