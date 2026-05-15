use crate::config::settings::{AppSettings, Preset};
use crate::i18n;

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) -> bool {
    let mut should_start_server = false;

    ui.heading(i18n::t(i18n::Key::SectionPresets, lang));
    ui.separator();

    // 帮助提示
    ui.small(i18n::t(i18n::Key::HintPresetHelp, lang));
    ui.add_space(8.0);

    // 保存预设区域
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelPresetName, lang));
        ui.text_edit_singleline(&mut settings.new_preset_name);
        if ui.button(i18n::t(i18n::Key::BtnSavePreset, lang)).clicked() {
            let trimmed = settings.new_preset_name.trim().to_string();
            if !trimmed.is_empty() {
                // 检查是否已存在同名预设
                let exists = settings.presets.iter().any(|p| p.name == trimmed);
                if !exists {
                    let preset = Preset::from_settings(settings, trimmed);
                    settings.presets.push(preset);
                } else {
                    // 覆盖现有预设
                    if let Some(idx) = settings.presets.iter().position(|p| p.name == trimmed) {
                        let new_preset = Preset::from_settings(settings, trimmed);
                        settings.presets[idx] = new_preset;
                    }
                }
                // 保存后清空输入框
                settings.new_preset_name.clear();
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
            let mut delete_index: Option<usize> = None;
            let mut auto_start_preset: Option<String> = None;

            for (i, preset) in settings.presets.iter().enumerate() {
                ui.horizontal(|ui| {
                    // 预设名称（可点击加载）
                    if ui.selectable_label(false, format!("📦 {}", preset.name)).clicked() {
                        load_index = Some(i);
                    }

                    // 应用按钮 - 加载配置到表单，不自动启动服务
                    if ui.small_button(i18n::t(i18n::Key::BtnApplyPreset, lang)).clicked() {
                        load_index = Some(i);
                    }

                     // 自启动预设勾选框（单选）
                    let mut is_auto = settings.auto_start_preset_name
                        .as_ref()
                        .is_some_and(|name| *name == preset.name);
                    if ui.checkbox(&mut is_auto, i18n::t(i18n::Key::CheckboxAutoStartPreset, lang)).changed() {
                        if is_auto {
                            auto_start_preset = Some(preset.name.clone());
                        } else if settings.auto_start_preset_name.as_ref() == Some(&preset.name) {
                            // 取消勾选时清除自启动预设标记
                            settings.auto_start_preset_name = None;
                        }
                    }

                    // 重命名按钮 - 使用 settings 持久化状态
                    if ui.small_button(i18n::t(i18n::Key::BtnRenamePreset, lang)).clicked() {
                        settings.rename_preset_index = Some(i);
                        settings.rename_preset_new_name = preset.name.clone();
                    }

                    // 删除按钮
                    if ui.small_button(i18n::t(i18n::Key::BtnDeletePreset, lang)).clicked() {
                        delete_index = Some(i);
                    }
                });
                ui.separator();
            }

            // 执行自启动预设设置
            if let Some(name) = auto_start_preset {
                settings.auto_start_preset_name = Some(name);
            }

            // 执行加载
            if let Some(idx) = load_index {
                if idx < settings.presets.len() {
                    let preset = settings.presets[idx].clone();
                    preset.apply_to(settings);
                }
            }

            // 检查是否需要启动 Server
            should_start_server = load_index.map(|idx|
                idx < settings.presets.len()
                    && settings.auto_start_preset_name.as_ref() == Some(&settings.presets[idx].name)
            ).unwrap_or(false);

            // 执行删除
            if let Some(idx) = delete_index {
                if idx < settings.presets.len() {
                    settings.presets.remove(idx);
                    // 如果删除的是正在重命名的预设，清空重命名状态
                    if let Some(rename_idx) = settings.rename_preset_index {
                        if rename_idx >= settings.presets.len() {
                            settings.rename_preset_index = None;
                            settings.rename_preset_new_name.clear();
                        }
                    }
                }
            }

            // 重命名弹窗 - 使用 settings 持久化输入框状态
            if let Some(idx) = settings.rename_preset_index {
                if idx < settings.presets.len() {
                    egui::Window::new(i18n::t(i18n::Key::BtnRenamePreset, lang))
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ui.ctx(), |ui| {
                            ui.label(i18n::t(i18n::Key::LabelPresetName, lang));
                            ui.text_edit_singleline(&mut settings.rename_preset_new_name);
                            ui.horizontal(|ui| {
                                if ui.button("确认").clicked() {
                                    let trimmed = settings.rename_preset_new_name.trim().to_string();
                                    if !trimmed.is_empty() {
                                        settings.presets[idx].name = trimmed;
                                    }
                                    settings.rename_preset_index = None;
                                    settings.rename_preset_new_name.clear();
                                }
                                if ui.button("取消").clicked() {
                                    settings.rename_preset_index = None;
                                    settings.rename_preset_new_name.clear();
                                }
                            });
                        });
                }
            }
        });
    }

    should_start_server
}
