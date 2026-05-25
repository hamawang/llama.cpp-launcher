use crate::config::settings::{AppSettings, GpuLayersMode};
use crate::i18n;
use crate::kv_cache;

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) {
    ui.heading(i18n::t(i18n::Key::PanelParamsTitle, lang));
    ui.separator();

    // Server 可启动判断（上下文长度按钮和 KV 缓存计算共用）
    let server_path_valid = settings.server_path
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(|name| name == "llama-server.exe");
    let can_start = server_path_valid && !settings.model_path.as_os_str().is_empty();

    // 上下文长度 (k) + 显存最大可用上下文按钮
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelNCtx, lang));
        ui.add(
            egui::DragValue::new(&mut settings.n_ctx)
                .range(1..=1024) // 1k ~ 1024k
                .speed(1),
        );
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
    });
    if ui.add_enabled(can_start, egui::Button::new(i18n::t(i18n::Key::BtnSetMaxContextVram, lang))).clicked() {
        match kv_cache::calc_max_context_facade(settings) {
            Ok(val) => settings.n_ctx = val,
            Err(e) => log::warn!("[params_panel] calc_max_context 失败: {}", e),
        }
    }

    // 最大批次大小 (--batch-size) (k)
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelBatchSize, lang));
        ui.add(
            egui::DragValue::new(&mut settings.batch_size)
                .range(1..=16)
                .speed(1),
        ); // 1k ~ 16k
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
    });

    // 最大物理批次大小 (--ubatch-size) (k)
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelUBatchSize, lang));
        ui.add(
            egui::DragValue::new(&mut settings.ubatch_size)
                .range(0.5..=16.0)
                .speed(0.5),
        ); // 0.5k ~ 16k, 步进 0.5
        ui.label("k");
        ui.small(i18n::t(i18n::Key::HintKUnit, lang));
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
    });

    // KV 缓存空间计算按钮
    ui.horizontal(|ui| {
        if ui.add_enabled(can_start, egui::Button::new(i18n::t(i18n::Key::BtnCalcKvCache, lang))).clicked() {
            settings.kv_cache_result = match kv_cache::calc_and_format(settings) {
                Ok(result) => Some(format!("{} {}", i18n::t(i18n::Key::LabelKvCacheResult, lang), result)),
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
        ui.add(egui::Slider::new(&mut settings.temperature, 0.0..=2.0).smallest_positive(0.01));
        ui.label(format!("{:.2}", settings.temperature));
    });

    // top_p
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTopP, lang));
        ui.add(egui::Slider::new(&mut settings.top_p, 0.0..=1.0).smallest_positive(0.01));
        ui.label(format!("{:.2}", settings.top_p));
    });

    // top_k
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTopK, lang));
        ui.add(egui::DragValue::new(&mut settings.top_k).range(0..=1000));
    });

    // 重复惩罚
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelRepeatPenalty, lang));
        ui.add(egui::Slider::new(&mut settings.repeat_penalty, 0.0..=2.0).smallest_positive(0.01));
        ui.label(format!("{:.2}", settings.repeat_penalty));
    });

    // 存在惩罚
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelPresencePenalty, lang));
        ui.add(
            egui::Slider::new(&mut settings.presence_penalty, -2.0..=2.0).smallest_positive(0.01),
        );
        ui.label(format!("{:.2}", settings.presence_penalty));
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
    });

    // 锁定内存
    ui.checkbox(
        &mut settings.kv_mlock,
        i18n::t(i18n::Key::CheckboxKvMlock, lang),
    );

    // 内存映射
    ui.checkbox(
        &mut settings.kv_mmap,
        i18n::t(i18n::Key::CheckboxKvMmap, lang),
    );

    // 统一键值缓存
    ui.checkbox(
        &mut settings.kv_unified,
        i18n::t(i18n::Key::CheckboxKvUnified, lang),
    );

    // 完整滑动窗口
    ui.checkbox(
        &mut settings.swa_full,
        i18n::t(i18n::Key::CheckboxSwaFull, lang),
    );

    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionGpuDevice, lang));
    ui.separator();

    // GPU 层数（统一使用拖拽输入框，默认值 99）
    let mut gpu_layers = match settings.gpu_layers_mode {
        GpuLayersMode::Auto | GpuLayersMode::All => 99usize,
        GpuLayersMode::Manual(n) => n,
    };

    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelGpuDevice, lang));
        if ui
            .add(egui::DragValue::new(&mut gpu_layers).range(0..=256))
            .changed()
        {
            settings.gpu_layers_mode = GpuLayersMode::Manual(gpu_layers);
        }
    });

    // 设备列表
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelRpcDevice, lang));
        ui.text_edit_singleline(&mut settings.gpu_device);
        ui.small(i18n::t(i18n::Key::HintRpcDevice, lang));
    });

    // 拆分模式
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelSplitMode, lang));
        let modes = ["none", "layer", "row", "tensor"];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for mode in &modes {
                let selected = settings.split_mode == *mode;
                if ui.selectable_label(selected, *mode).clicked() {
                    settings.split_mode = mode.to_string();
                }
            }
        });
        ui.small(i18n::t(i18n::Key::HintSplitMode, lang));
    });

    // 张量拆分比例
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelTensorSplit, lang));
        ui.text_edit_singleline(&mut settings.tensor_split);
        ui.small(i18n::t(i18n::Key::HintTensorSplit, lang));
    });

    // CPU MoE（与 RPC 模式一致的缩进样式）
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut settings.cpu_moe,
            i18n::t(i18n::Key::CheckboxCpuMoe, lang),
        );
        ui.small(i18n::t(i18n::Key::HintCpuMoe, lang));
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
    });

    // 最大推测数量 --spec-draft-n-max（DragValue）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftNMaxLabel, lang));
        ui.add(egui::DragValue::new(&mut settings.spec_draft_n_max).range(0..=64));
    });

    // 最小推测数量 --spec-draft-n-min（DragValue）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftNMinLabel, lang));
        ui.add(egui::DragValue::new(&mut settings.spec_draft_n_min).range(0..=32));
    });

    // 信任度 --spec-draft-p-min（Slider）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftPMinLabel, lang));
        ui.add(
            egui::Slider::new(&mut settings.spec_draft_p_min, 0.0..=1.0).smallest_positive(0.01),
        );
        ui.label(format!("{:.2}", settings.spec_draft_p_min));
    });

    // 分裂概率 --spec-draft-p-split（Slider）
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::SpecDraftPSplitLabel, lang));
        ui.add(
            egui::Slider::new(&mut settings.spec_draft_p_split, 0.0..=1.0).smallest_positive(0.01),
        );
        ui.label(format!("{:.2}", settings.spec_draft_p_split));
    });
}
