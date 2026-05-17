use crate::config::settings::AppSettings;
use crate::engine::server::{LogLevel, ServerManager};
use crate::i18n;

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, server: &mut ServerManager, lang: &i18n::Language) {
    ui.heading(i18n::t(i18n::Key::PanelLogTitle, lang));
    ui.separator();

    ui.horizontal(|ui| {
        if ui.small_button(i18n::t(i18n::Key::BtnClearLogs, lang)).clicked() {
            server.clear_logs();
        }
        ui.checkbox(&mut settings.auto_scroll_logs, i18n::t(i18n::Key::CheckboxAutoScroll, lang));
        ui.add_space(8.0);
        ui.label(i18n::t(i18n::Key::LabelMaxLogLines, lang));
        ui.add(egui::DragValue::new(&mut settings.max_log_lines).range(-1..=10000));
        ui.small(i18n::t(i18n::Key::HintLogSession, lang));
    });

    ui.add_space(8.0);

    // 预填充进度条（0-100%），仅在进度 > 0 时显示
    let progress = server.progress();
    if progress > 0.0 {
        let pct = (progress * 100.0).round() as u32;
        let label = format!("{}: {}/100%", i18n::t(i18n::Key::LabelPreFillProgress, lang), pct);
        ui.add(
            egui::ProgressBar::new(progress)
                .text(&label),
        );
        ui.add_space(4.0);
    }

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .id_salt("log_scroll_area")
        .stick_to_bottom(settings.auto_scroll_logs)  // 根据复选框控制自动滚动
        .show(ui, |ui| {
            let mut logs = server.logs();
            // 根据设置截断日志行数 (-1 表示全部保留)
            if settings.max_log_lines > 0 && logs.len() > settings.max_log_lines as usize {
                let start_index = logs.len() - settings.max_log_lines as usize;
                logs.drain(..start_index);
            }
            
            if logs.is_empty() {
                ui.add_space(20.0);
                ui.horizontal_centered(|ui| {
                    ui.colored_label(egui::Color32::GRAY, i18n::t(i18n::Key::HintNoLogs, lang));
                });
            } else {
                for entry in &logs {
                    let prefix = match entry.level {
                        LogLevel::Info => "",
                        LogLevel::Warn => "⚠ ",
                        LogLevel::Error => "✖ ",
                    };

                    let text = format!("{}{}", prefix, entry.text);

                    ui.horizontal_wrapped(|ui| {
                        match entry.level {
                            LogLevel::Info => {
                                ui.colored_label(egui::Color32::BLACK, &text);
                            }
                            LogLevel::Warn => {
                                egui::Frame::default()
                                    .fill(egui::Color32::from_rgb(80, 80, 80))
                                    .inner_margin(egui::Margin::same(4.0))
                                    .show(ui, |ui| {
                                        ui.colored_label(egui::Color32::YELLOW, &text);
                                    });
                            }
                            LogLevel::Error => {
                                ui.colored_label(egui::Color32::RED, &text);
                            }
                        }
                    });
                }
            }
        });
    
}
