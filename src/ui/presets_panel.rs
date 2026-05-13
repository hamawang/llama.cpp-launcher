use crate::config::settings::{AppSettings, Preset};
use crate::i18n;

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) {
    ui.heading(i18n::t(i18n::Key::SectionPresets, lang));
    ui.separator();

    // 帮助提示
    ui.small(i18n::t(i18n::Key::HintPresetHelp, lang));
    ui.add_space(8.0);

    // 保存预设区域
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelPresetName, lang));
        let mut name = String::new();
        ui.text_edit_singleline(&mut name);
        if ui.button(i18n::t(i18n::Key::BtnSavePreset, lang)).clicked() {
            if !name.trim().is_empty() {
                let name = name.trim().to_string();
                // 检查是否已存在同名预设
                let exists = settings.presets.iter().any(|p| p.name == name);
                if !exists {
                    let preset = Preset::from_settings(settings, name);
                    settings.presets.push(preset);
                } else {
                    // 覆盖现有预设 - 先找到索引
                    if let Some(idx) = settings.presets.iter().position(|p| p.name == name) {
                        let new_preset = Preset::from_settings(settings, name);
                        settings.presets[idx] = new_preset;
                    }
                }
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();

      // 预设列表
    if settings.presets.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(i18n::t(i18n::Key::HintNoPresets, lang));
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 先收集需要操作的信息，避免借用冲突
            let mut load_index: Option<usize> = None;
            let mut rename_index: Option<usize> = None;
            let mut delete_index: Option<usize> = None;

            for (i, preset) in settings.presets.iter().enumerate() {
                ui.horizontal(|ui| {
                    // 预设名称（可点击加载）
                    if ui.selectable_label(false, format!("📦 {}", preset.name)).clicked() {
                        load_index = Some(i);
                    }

                    // 重命名按钮
                    if ui.small_button(i18n::t(i18n::Key::BtnRenamePreset, lang)).clicked() {
                        rename_index = Some(i);
                    }

                    // 删除按钮
                    if ui.small_button(i18n::t(i18n::Key::BtnDeletePreset, lang)).clicked() {
                        delete_index = Some(i);
                    }
                });
                ui.separator();
            }

            // 执行操作
            if let Some(idx) = load_index {
                if idx < settings.presets.len() {
                    let preset = settings.presets[idx].clone();
                    preset.apply_to(settings);
                }
            }

            if let Some(idx) = rename_index {
                if idx < settings.presets.len() {
                    let mut new_name = settings.presets[idx].name.clone();
                    egui::Window::new("重命名预设")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ui.ctx(), |ui| {
                            ui.label(i18n::t(i18n::Key::LabelPresetName, lang));
                            ui.text_edit_singleline(&mut new_name);
                            if ui.button("确认").clicked() {
                                if !new_name.trim().is_empty() {
                                    settings.presets[idx].name = new_name.trim().to_string();
                                }
                            }
                        });
                }
            }

            if let Some(idx) = delete_index {
                if idx < settings.presets.len() {
                    settings.presets.remove(idx);
                }
            }
        });
    }
}
