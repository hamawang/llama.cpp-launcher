use crate::config::settings::{AppSettings, GpuLayersMode, Preset};
use crate::i18n;
use serde::{Deserialize, Serialize};

/// 导出/导入的“参数面板”专用结构（不包含 Server/RPC/模型路径等）
#[derive(Serialize, Deserialize)]
struct ParamsExport {
    context: usize,
    batch_size: usize,
    ubatch_size: f32,
    temperature: f32,
    top_p: f32,
    top_k: i32,
    repeat_penalty: f32,
    presence_penalty: f32,
    flash_attn: String,

    // 推测解码
    spec_type: String,
    spec_draft_n_max: usize,
    spec_draft_n_min: usize,
    spec_draft_p_min: f32,
    spec_draft_p_split: f32,

    // KV 缓存
    kv_offload: bool,
    cache_type_k: String,
    cache_type_v: String,
    kv_mlock: bool,
    kv_mmap: bool,
    kv_unified: bool,
    swa_full: bool,

    // GPU/设备分配
    gpu_device: String,
    gpu_layers_mode: GpuLayersMode,
    split_mode: String,
    tensor_split: String,
    cpu_moe: bool,
    n_cpu_moe: usize,
}

impl ParamsExport {
    fn from_settings(s: &AppSettings) -> Self {
        Self {
            context: s.context,
            batch_size: s.batch_size,
            ubatch_size: s.ubatch_size,
            temperature: s.temperature,
            top_p: s.top_p,
            top_k: s.top_k,
            repeat_penalty: s.repeat_penalty,
            presence_penalty: s.presence_penalty,
            flash_attn: s.flash_attn.clone(),

            spec_type: s.spec_type.clone(),
            spec_draft_n_max: s.spec_draft_n_max,
            spec_draft_n_min: s.spec_draft_n_min,
            spec_draft_p_min: s.spec_draft_p_min,
            spec_draft_p_split: s.spec_draft_p_split,

            kv_offload: s.kv_offload,
            cache_type_k: s.cache_type_k.clone(),
            cache_type_v: s.cache_type_v.clone(),
            kv_mlock: s.kv_mlock,
            kv_mmap: s.kv_mmap,
            kv_unified: s.kv_unified,
            swa_full: s.swa_full,

            gpu_device: s.gpu_device.clone(),
            gpu_layers_mode: s.gpu_layers_mode,
            split_mode: s.split_mode.clone(),
            tensor_split: s.tensor_split.clone(),
            cpu_moe: s.cpu_moe,
            n_cpu_moe: s.n_cpu_moe,
        }
    }

    fn apply_to(self, s: &mut AppSettings) {
        s.context = self.context;
        s.batch_size = self.batch_size;
        s.ubatch_size = self.ubatch_size;
        s.temperature = self.temperature;
        s.top_p = self.top_p;
        s.top_k = self.top_k;
        s.repeat_penalty = self.repeat_penalty;
        s.presence_penalty = self.presence_penalty;
        s.flash_attn = self.flash_attn;

        s.spec_type = self.spec_type;
        s.spec_draft_n_max = self.spec_draft_n_max;
        s.spec_draft_n_min = self.spec_draft_n_min;
        s.spec_draft_p_min = self.spec_draft_p_min;
        s.spec_draft_p_split = self.spec_draft_p_split;

        s.kv_offload = self.kv_offload;
        s.cache_type_k = self.cache_type_k;
        s.cache_type_v = self.cache_type_v;
        s.kv_mlock = self.kv_mlock;
        s.kv_mmap = self.kv_mmap;
        s.kv_unified = self.kv_unified;
        s.swa_full = self.swa_full;

        s.gpu_device = self.gpu_device;
        s.gpu_layers_mode = self.gpu_layers_mode;
        s.split_mode = self.split_mode;
        s.tensor_split = self.tensor_split;
        s.cpu_moe = self.cpu_moe;
        s.n_cpu_moe = self.n_cpu_moe;
    }
}

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) -> bool {
    let mut should_start_server = false;

    ui.heading(i18n::t(i18n::Key::SectionPresets, lang));
    ui.separator();

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

        // 导出参数预设按钮（仅导出参数面板相关字段）
        if ui
            .small_button(i18n::t(i18n::Key::BtnExportParams, lang))
            .clicked()
        {
            let params = ParamsExport::from_settings(settings);
            let json = match serde_json::to_string_pretty(&params) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[导出参数预设] 序列化失败: {}", e);
                    return;
                }
            };

            // 使用 rfd save_file，建议文件名固定前缀
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("llama_cpp_launcher_parameter_export.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, &json) {
                    eprintln!("[导出参数预设] 写入失败: {}", e);
                }
            }
        }

        // 导入参数预设按钮（立即应用到参数面板）
        if ui
            .small_button(i18n::t(i18n::Key::BtnImportParams, lang))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                let content = match std::fs::read_to_string(&path) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[导入参数预设] 读取失败: {}", e);
                        return;
                    }
                };

                // 反序列化为 ParamsExport（允许字段不完全匹配时宽容处理）
                match serde_json::from_str::<ParamsExport>(&content) {
                    Ok(params) => {
                        params.apply_to(settings);
                    }
                    Err(e) => {
                        eprintln!("[导入参数预设] 解析失败: {}", e);
                    }
                }
            }
        }
    });

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
                    if ui
                        .selectable_label(false, format!("📦 {}", preset.name))
                        .clicked()
                    {
                        load_index = Some(i);
                    }

                    // 应用按钮 - 加载配置到表单，不自动启动服务
                    if ui
                        .small_button(i18n::t(i18n::Key::BtnApplyPreset, lang))
                        .clicked()
                    {
                        load_index = Some(i);
                    }

                    // 自启动预设勾选框（单选）
                    let mut is_auto = settings
                        .auto_start_preset_name
                        .as_ref()
                        .is_some_and(|name| *name == preset.name);
                    if ui
                        .checkbox(
                            &mut is_auto,
                            i18n::t(i18n::Key::CheckboxAutoStartPreset, lang),
                        )
                        .changed()
                    {
                        if is_auto {
                            auto_start_preset = Some(preset.name.clone());
                        } else if settings.auto_start_preset_name.as_ref() == Some(&preset.name) {
                            // 取消勾选时清除自启动预设标记
                            settings.auto_start_preset_name = None;
                        }
                    }

                    // 重命名按钮 - 使用 settings 持久化状态
                    if ui
                        .small_button(i18n::t(i18n::Key::BtnRenamePreset, lang))
                        .clicked()
                    {
                        settings.rename_preset_index = Some(i);
                        settings.rename_preset_new_name = preset.name.clone();
                    }

                    // 删除按钮
                    if ui
                        .small_button(i18n::t(i18n::Key::BtnDeletePreset, lang))
                        .clicked()
                    {
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
            should_start_server = load_index
                .map(|idx| {
                    idx < settings.presets.len()
                        && settings.auto_start_preset_name.as_ref()
                            == Some(&settings.presets[idx].name)
                })
                .unwrap_or(false);

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
                        .fixed_size([150.0, 0.0])
                        .show(ui.ctx(), |ui| {
                            ui.label(i18n::t(i18n::Key::LabelPresetName, lang));
                            ui.text_edit_singleline(&mut settings.rename_preset_new_name);
                            ui.horizontal(|ui| {
                                if ui.button("确认").clicked() {
                                    let trimmed =
                                        settings.rename_preset_new_name.trim().to_string();
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
