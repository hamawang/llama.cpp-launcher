use crate::config::settings::{is_server_binary_name, AppSettings, GpuLayersMode};
use crate::i18n;
use crate::kv_cache;
use crate::ui::helper;

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) {
    ui.heading(i18n::t(i18n::Key::PanelParamsTitle, lang));
    ui.separator();

    // Server 可启动判断（上下文长度按钮和 KV 缓存计算共用）
    let server_path_valid = settings
        .server_path
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(is_server_binary_name);
    let can_start = server_path_valid && !settings.model_path.as_os_str().is_empty();

    // 上下文长度 (k) + 显存最大可用上下文按钮
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelNCtx, lang));
        ui.add(
            egui::DragValue::new(&mut settings.context)
                .range(1..=1024) // 1k ~ 1024k
                .speed(1),
        );
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpNCtx, lang));
    });
    if ui
        .add_enabled(
            can_start,
            egui::Button::new(i18n::t(i18n::Key::BtnSetMaxContextVram, lang)),
        )
        .clicked()
    {
        match kv_cache::calc_max_context_facade(settings) {
            Ok(val) => settings.context = val,
            Err(e) => log::warn!("[params_panel] calc_max_context 失败: {}", e),
        }
    }

    // 批次大小 (--batch-size) (k)
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelBatchSize, lang));
        ui.add(
            egui::DragValue::new(&mut settings.batch_size)
                .range(1..=16)
                .speed(1),
        ); // 1k ~ 16k
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpBatchSize, lang));
    });

    // 物理批次大小 (--ubatch-size) (k)
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelUBatchSize, lang));
        ui.add(
            egui::DragValue::new(&mut settings.ubatch_size)
                .range(0.5..=16.0)
                .speed(0.5),
        ); // 0.5k ~ 16k, 步进 0.5
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpUBatchSize, lang));
    });

    // 会话超时设置 (--timeout) (秒)
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelSessionTimeout, lang));
        ui.add(
            egui::DragValue::new(&mut settings.session_timeout)
                .range(60..=3600)
                .speed(10),
        ); // 60~3600秒，步进10
        ui.label(i18n::t(i18n::Key::HintSUnit, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSessionTimeout, lang));
    });

    // KV 缓存比例（DragValue）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelKvCacheRatio, lang));
        ui.add(
            egui::DragValue::new(&mut settings.kv_cache_ratio)
                .range(0.0..=1.0)
                .speed(0.01),
        );
        ui.label(format!("{:.2}", settings.kv_cache_ratio));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpKvCacheRatio, lang));
    });

    // KV 缓存空间计算按钮
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                can_start,
                egui::Button::new(i18n::t(i18n::Key::BtnCalcKvCache, lang)),
            )
            .clicked()
        {
            settings.kv_cache_result = match kv_cache::calc_and_format(settings) {
                Ok(result) => Some(format!(
                    "{} {}",
                    i18n::t(i18n::Key::LabelKvCacheResult, lang),
                    result
                )),
                Err(e) => Some(format!("⚠ {}", e)),
            };
        }

        if let Some(ref result) = settings.kv_cache_result {
            ui.small(egui::RichText::new(result).weak());
        }
    });

    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionSampling, lang));
    ui.separator();

    // 温度
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTemperature, lang));
        ui.add(
            egui::Slider::new(&mut settings.temperature, 0.0..=2.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.temperature));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpTemperature, lang));
        ui.checkbox(
            &mut settings.ignore_temperature,
            i18n::t(i18n::Key::CheckboxIgnoreTemperature, lang),
        );
    });

    // top_p
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTopP, lang));
        ui.add(
            egui::Slider::new(&mut settings.top_p, 0.0..=1.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.top_p));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpTopP, lang));
        ui.checkbox(
            &mut settings.ignore_top_p,
            i18n::t(i18n::Key::CheckboxIgnoreTopP, lang),
        );
    });

    // top_k
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTopK, lang));
        ui.add(egui::DragValue::new(&mut settings.top_k).range(0..=1000));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpTopK, lang));
        ui.checkbox(
            &mut settings.ignore_top_k,
            i18n::t(i18n::Key::CheckboxIgnoreTopK, lang),
        );
    });

    // 重复惩罚
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelRepeatPenalty, lang));
        ui.add(
            egui::Slider::new(&mut settings.repeat_penalty, 0.0..=2.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.repeat_penalty));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpRepeatPenalty, lang));
        ui.checkbox(
            &mut settings.ignore_repeat_penalty,
            i18n::t(i18n::Key::CheckboxIgnoreRepeatPenalty, lang),
        );
    });

    // 存在惩罚
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelPresencePenalty, lang));
        ui.add(
            egui::Slider::new(&mut settings.presence_penalty, -2.0..=2.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.presence_penalty));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpPresencePenalty, lang));
        ui.checkbox(
            &mut settings.ignore_presence_penalty,
            i18n::t(i18n::Key::CheckboxIgnorePresencePenalty, lang),
        );
    });

    // Flash Attention（国际化选项）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelFlashAttn, lang));
        let fa_modes = [
            ("on", i18n::Key::FaModeOn),
            ("off", i18n::Key::FaModeOff),
            ("auto", i18n::Key::FaModeAuto),
        ];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (value, key) in &fa_modes {
                let label = i18n::t(*key, lang);
                let selected = settings.flash_attn == *value;
                if ui.selectable_label(selected, label).clicked() {
                    settings.flash_attn = value.to_string();
                }
            }
        });
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpFlashAttn, lang));
    });

    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionKvCache, lang));
    ui.separator();

    // K/V 缓存卸载
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.kv_offload,
            i18n::t(i18n::Key::CheckboxKvOffload, lang),
        );
        ui.small(i18n::t(i18n::Key::HintKvOffload, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpKvOffload, lang));
    });

    // K 缓存类型
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelCacheTypeK, lang));
        let k_types = [
            "f32", "f16", "bf16", "q8_0", "q4_0", "q4_1", "iq4_nl", "q5_0", "q5_1",
        ];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for k_type in &k_types {
                let selected = settings.cache_type_k == *k_type;
                if ui.selectable_label(selected, *k_type).clicked() {
                    settings.cache_type_k = k_type.to_string();
                }
            }
        });
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpCacheTypeK, lang));
    });

    // V 缓存类型
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelCacheTypeV, lang));
        let v_types = [
            "f32", "f16", "bf16", "q8_0", "q4_0", "q4_1", "iq4_nl", "q5_0", "q5_1",
        ];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for v_type in &v_types {
                let selected = settings.cache_type_v == *v_type;
                if ui.selectable_label(selected, *v_type).clicked() {
                    settings.cache_type_v = v_type.to_string();
                }
            }
        });
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpCacheTypeV, lang));
    });

    // 锁定内存
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.kv_mlock,
            i18n::t(i18n::Key::CheckboxKvMlock, lang),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpKvMlock, lang));
    });

    // 内存映射
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.kv_mmap,
            i18n::t(i18n::Key::CheckboxKvMmap, lang),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpKvMmap, lang));
    });

    // 统一键值缓存
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.kv_unified,
            i18n::t(i18n::Key::CheckboxKvUnified, lang),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpKvUnified, lang));
    });

    // 完整滑动窗口
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.swa_full,
            i18n::t(i18n::Key::CheckboxSwaFull, lang),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSwaFull, lang));
    });

    // 上下文检查点
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelCtxCheckpoints, lang));
        ui.add(
            egui::DragValue::new(&mut settings.ctx_checkpoints)
                .range(1..=256)
                .speed(1),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpCtxCheckpoints, lang));
    });

    // 最小检查点步长
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelCheckpointMinStep, lang));
        ui.add(
            egui::DragValue::new(&mut settings.checkpoint_min_step)
                .range(64..=4096)
                .speed(64),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpCheckpointMinStep, lang));
    });

    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionGpuDevice, lang));
    ui.separator();

    // GPU 层数（可选择标签 + 手动指定复选框）
    let mut manual_gpu_layers = matches!(settings.gpu_layers_mode, GpuLayersMode::Manual(_));
    let mut gpu_layers = match settings.gpu_layers_mode {
        GpuLayersMode::Auto => 0usize,
        GpuLayersMode::All => 256usize,
        GpuLayersMode::Manual(n) => n,
    };

    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelGpuDevice, lang));
        let gpu_modes = [
            (i18n::Key::GpuModeAuto, GpuLayersMode::Auto),
            (i18n::Key::GpuModeAll, GpuLayersMode::All),
        ];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (label_key, mode) in &gpu_modes {
                let selected = match &settings.gpu_layers_mode {
                    GpuLayersMode::Auto => matches!(mode, GpuLayersMode::Auto),
                    GpuLayersMode::All => matches!(mode, GpuLayersMode::All),
                    GpuLayersMode::Manual(_) => false,
                };
                let label = i18n::t(*label_key, lang);
                if ui.selectable_label(selected, label).clicked() {
                    settings.gpu_layers_mode = *mode;
                    manual_gpu_layers = false;
                }
            }
        });
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpGpuDevice, lang));
    });

    // 手动指定 GPU 层数
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut manual_gpu_layers,
            i18n::t(i18n::Key::CheckboxManualGpuLayers, lang),
        );
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpGpuDevice, lang));
    });
    if manual_gpu_layers {
        ui.indent("manual_gpu_layers_options", |ui| {
            ui.horizontal(|ui| {
                ui.label(i18n::t(i18n::Key::LabelGpuDevice, lang));
                ui.add(egui::DragValue::new(&mut gpu_layers).range(0..=256));
                ui.small(i18n::t(i18n::Key::HintGpuDevice, lang));
            });
        });
        settings.gpu_layers_mode = GpuLayersMode::Manual(gpu_layers);
    }

    // 拆分模式
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelSplitMode, lang));
        let modes: [(&str, i18n::Key); 3] = [
            ("none", i18n::Key::SplitModeNone),
            ("layer", i18n::Key::SplitModeLayer),
            ("tensor", i18n::Key::SplitModeTensor),
        ];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (value, label_key) in &modes {
                let selected = settings.split_mode == *value;
                if ui
                    .selectable_label(selected, i18n::t(*label_key, lang))
                    .clicked()
                {
                    settings.split_mode = value.to_string();
                }
            }
        });
        ui.small(i18n::t(i18n::Key::HintSplitMode, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSplitMode, lang));
    });

    // 张量拆分比例
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTensorSplit, lang));
        ui.text_edit_singleline(&mut settings.tensor_split);
        ui.small(i18n::t(i18n::Key::HintTensorSplit, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpTensorSplit, lang));
    });

    // CPU MoE（与 RPC 模式一致的缩进样式）
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.cpu_moe,
            i18n::t(i18n::Key::CheckboxCpuMoe, lang),
        );
        ui.small(i18n::t(i18n::Key::HintCpuMoe, lang));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpCpuMoe, lang));
    });
    if settings.cpu_moe {
        ui.indent("cpu_moe_options", |ui| {
            ui.horizontal(|ui| {
                ui.label(i18n::t(i18n::Key::LabelNCpuMoe, lang));
                ui.add(egui::DragValue::new(&mut settings.n_cpu_moe).range(0..=256));
                ui.small(i18n::t(i18n::Key::HintNCpuMoe, lang));
            });
        });
    }

    // 推测解码（Speculative Decoding）- GPU/设备分配下方追加的新设置
    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionSpecDecoding, lang));
    ui.separator();

    // 算法类型 --spec-type（与拆分模式相同样式）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecTypeLabel, lang));

        let spec_options = [
            "none",
            "draft-simple",
            "draft-eagle3",
            "draft-mtp",
            "ngram-simple",
            "ngram-map-k",
            "ngram-map-k4v",
            "ngram-mod",
            "ngram-cache",
            "dflash",
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for opt in &spec_options[..] {
                let selected = settings.spec_type == *opt;
                if ui.selectable_label(selected, *opt).clicked() {
                    settings.spec_type = opt.to_string();
                }
            }
        });
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSpecType, lang));
    });

    // 最大推测数量 --spec-draft-n-max（DragValue）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftNMaxLabel, lang));
        ui.add(egui::DragValue::new(&mut settings.spec_draft_n_max).range(0..=64));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSpecDraftNMax, lang));
    });

    // 最小推测数量 --spec-draft-n-min（DragValue）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftNMinLabel, lang));
        ui.add(egui::DragValue::new(&mut settings.spec_draft_n_min).range(0..=32));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSpecDraftNMin, lang));
    });

    // 信任度 --spec-draft-p-min（Slider）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftPMinLabel, lang));
        ui.add(
            egui::Slider::new(&mut settings.spec_draft_p_min, 0.0..=1.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.spec_draft_p_min));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSpecDraftPMin, lang));
    });

    // 分裂概率 --spec-draft-p-split（Slider）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftPSplitLabel, lang));
        ui.add(
            egui::Slider::new(&mut settings.spec_draft_p_split, 0.0..=1.0)
                .smallest_positive(0.01)
                .custom_formatter(|v, _| format!("{:.2}", v)),
        );
        ui.label(format!("{:.2}", settings.spec_draft_p_split));
        helper::help_button_inline(ui, i18n::t(i18n::Key::HelpSpecDraftPSplit, lang));
    });
}
