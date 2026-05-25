#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

pub mod app;
pub mod config;
pub mod engine;
pub mod i18n;
pub mod kv_cache;
pub mod shortcut;
pub mod ui;
mod spacing_debugger;

use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};
use std::env;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// 文件日志写入开关（全局标志，由帮助菜单复选框控制）
static LOG_TO_FILE_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_log_to_file(enabled: bool) {
    LOG_TO_FILE_ENABLED.store(enabled, Ordering::Relaxed);
}

struct FileLogger {
    writer: Mutex<BufWriter<std::fs::File>>,
}

impl Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool { true }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && LOG_TO_FILE_ENABLED.load(Ordering::Relaxed) {
            let mut w = self.writer.lock().unwrap();
            writeln!(w, "[{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), record.args()).ok();
            w.flush().ok(); // 强制刷新，确保日志立即写入磁盘
        }
    }
    fn flush(&self) {
        self.writer.lock().unwrap().flush().ok();
    }
}

fn init_logger() {
    // 获取 exe 同级目录
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    
    // 生成带时间戳的日志文件名: llama_launcher_20260525_143000.log
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let log_path = exe_dir.join(format!("llama_launcher_{}.log", timestamp));

    // 打开或创建日志文件（追加模式）
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to create log file");

    let logger = FileLogger {
        writer: Mutex::new(BufWriter::new(file)),
    };

    // 初始化日志系统
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}

use app::LlamaLauncherApp;
use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

fn main() -> eframe::Result {
    init_logger();

    // 使用统一的主窗口尺寸
    let default_size = egui::vec2(1250.0, 800.0);

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(default_size)
        .with_title("llama.cpp launcher");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "llama.cpp launcher",
        options,
        Box::new(|cc| {
            // 配置 CJK 中文字体，解决中文乱码问题
            let mut fonts = FontDefinitions::default();
            load_cjk_fonts(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(LlamaLauncherApp::new(&cc)))
        }),
    )
}

/// 从系统字体目录加载 CJK 中文字体 (适配 egui 0.34: Arc<FontData>)
fn load_cjk_fonts(fonts: &mut FontDefinitions) {
    let cjk_proportional: Vec<&str> = if cfg!(target_os = "windows") {
        // Windows 中文字体文件路径
        vec![
            ("C:\\Windows\\Fonts\\msyh.ttc", "Microsoft YaHei"), // 微软雅黑
            ("C:\\Windows\\Fonts\\msyhbd.ttc", "Microsoft YaHei Bold"), // 微软雅黑粗体
            ("C:\\Windows\\Fonts\\simhei.ttf", "SimHei"),        // 黑体
            ("C:\\Windows\\Fonts\\simsun.ttc", "SimSun"),        // 宋体
        ]
            .into_iter()
            .filter_map(|(path, name)| {
                if let Ok(data) = std::fs::read(path) {
                    fonts
                        .font_data
                        .insert(name.to_string(), Arc::new(FontData::from_owned(data)));
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    } else if cfg!(target_os = "macos") {
        vec![
            ("/System/Library/Fonts/PingFang.ttc", "PingFang SC"),
            ("/System/Library/Fonts/STHeiti Lite.ttc", "STHeiti"),
            (
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "Arial Unicode",
            ),
        ]
            .into_iter()
            .filter_map(|(path, name)| {
                if let Ok(data) = std::fs::read(path) {
                    fonts
                        .font_data
                        .insert(name.to_string(), Arc::new(FontData::from_owned(data)));
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    } else {
        // Linux
        vec![
            (
                "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.ttf",
                "Noto Sans SC",
            ),
            (
                "/usr/share/fonts/opentype/noto/NotoSansSC-Regular.otf",
                "Noto Sans SC",
            ),
            (
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "WenQuanYi Micro Hei",
            ),
        ]
            .into_iter()
            .filter_map(|(path, name)| {
                if let Ok(data) = std::fs::read(path) {
                    fonts
                        .font_data
                        .insert(name.to_string(), Arc::new(FontData::from_owned(data)));
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    };

    // 将 CJK 字体添加到 Proportional 和 Monospace 家族，作为 fallback
    if !cjk_proportional.is_empty() {
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_insert_with(|| {
                let mut vec = Vec::new();
                vec.push("Ubuntu-Light".to_owned());
                vec
            })
            .extend(cjk_proportional.iter().map(|s| s.to_string()));

        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_insert_with(|| {
                let mut vec = Vec::new();
                vec.push("Hack".to_owned());
                vec
            })
            .extend(cjk_proportional.iter().map(|s| s.to_string()));
    }
}
