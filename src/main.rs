#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod config;
mod engine;
mod i18n;
mod ui;
mod shortcut;

use app::LlamaLunchApp;
use egui::{FontData, FontDefinitions, FontFamily};

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1100.0, 700.0))
            .with_title("llama.cpp lunch"),
        ..Default::default()
    };

    eframe::run_native(
        "llama.cpp lunch",
        options,
        Box::new(|cc| {
            // 配置 CJK 中文字体，解决中文乱码问题
            let mut fonts = FontDefinitions::default();
            load_cjk_fonts(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(LlamaLunchApp::new(&cc)))
        }),
    )
}

/// 从系统字体目录加载 CJK 中文字体
fn load_cjk_fonts(fonts: &mut FontDefinitions) {
    let cjk_proportional: Vec<&str> = if cfg!(target_os = "windows") {
        // Windows 中文字体文件路径
        vec![
            ("C:\\Windows\\Fonts\\msyh.ttc", "Microsoft YaHei"),       // 微软雅黑
            ("C:\\Windows\\Fonts\\msyhbd.ttc", "Microsoft YaHei Bold"), // 微软雅黑粗体
            ("C:\\Windows\\Fonts\\simhei.ttf", "SimHei"),               // 黑体
            ("C:\\Windows\\Fonts\\simsun.ttc", "SimSun"),               // 宋体
        ]
        .into_iter()
        .filter_map(|(path, name)| {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(name.to_string(), FontData::from_owned(data));
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
            ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", "Arial Unicode"),
        ]
        .into_iter()
        .filter_map(|(path, name)| {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(name.to_string(), FontData::from_owned(data));
                Some(name)
            } else {
                None
            }
        })
        .collect()
    } else {
        // Linux
        vec![
            ("/usr/share/fonts/truetype/noto/NotoSansSC-Regular.ttf", "Noto Sans SC"),
            ("/usr/share/fonts/opentype/noto/NotoSansSC-Regular.otf", "Noto Sans SC"),
            (
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "WenQuanYi Micro Hei",
            ),
        ]
        .into_iter()
        .filter_map(|(path, name)| {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(name.to_string(), FontData::from_owned(data));
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


