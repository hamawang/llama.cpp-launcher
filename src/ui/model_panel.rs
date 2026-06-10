use crate::config::settings::AppSettings;
use crate::i18n;

/// 自动检测模型文件夹
/// 检查应用所在目录中是否有 model 或 models 文件夹（不区分大小写）
fn auto_detect_model_dir() -> Option<std::path::PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();

    let dirs: Vec<_> = match std::fs::read_dir(&exe_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect(),
        Err(_) => return None,
    };

    // 优先匹配 "models"，其次 "model"
    let models_dir = dirs.iter().find(|e| {
        e.file_name()
            .to_string_lossy()
            .to_lowercase()
            .eq_ignore_ascii_case("models")
    });
    if let Some(dir) = models_dir {
        return Some(dir.path());
    }

    let model_dir = dirs.iter().find(|e| {
        e.file_name()
            .to_string_lossy()
            .to_lowercase()
            .eq_ignore_ascii_case("model")
    });
    if let Some(dir) = model_dir {
        return Some(dir.path());
    }

    None
}

/// 文件名解析为彩色标签（9 色方案）
fn parse_tags(filename: &str) -> Vec<(String, egui::Color32)> {
    let stem = filename.strip_suffix(".gguf").unwrap_or(filename);

    // 原有 5 色
    let purple = egui::Color32::from_rgb(180, 120, 255); // 参数量
    let orange = egui::Color32::from_rgb(255, 165, 0); // 量化类型
    let gray = egui::Color32::from_rgb(160, 160, 160); // 版本号
    let green = egui::Color32::from_rgb(100, 200, 100); // 训练方法
    let blue = egui::Color32::from_rgb(100, 150, 255); // 模型名称 (兜底)
                                                       // 新增 4 色
    let yellow = egui::Color32::from_rgb(255, 215, 0); // 精度
    let pink = egui::Color32::from_rgb(255, 100, 130); // LoRA/Adapter
    let brown = egui::Color32::from_rgb(205, 133, 63); // 上下文长度
    let cyan = egui::Color32::from_rgb(0, 210, 210); // 架构类型

    let mut tags = Vec::new();
    for part in stem.split('-') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        let color = if is_param_size(&lower) {
            // 🟣 参数量: 7b, 335m, 1.5b
            purple
        } else if is_quantization(&lower) {
            // 🟠 量化类型: q4_k_m, q8_0, iq4_nl, iq2_xs …
            orange
        } else if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
            // ⚫ 版本号: 3.1, 2
            gray
        } else if is_training_method(&lower) {
            // 🟢 训练方法: instruct/chat/sft/rlhf/dpo/orpo/grpo
            green
        } else if lower.contains("fp16")
            || lower.contains("bf16")
            || lower.contains("f32")
            || lower.contains("fp8")
        {
            // 🟡 精度: fp16, bf16, f32, fp8
            yellow
        } else if lower.contains("lora") || lower.contains("adapter") || lower.contains("delta") {
            // 🩷 LoRA/Adapter/Delta
            pink
        } else if is_context_length(&lower) {
            // 🟤 上下文长度: 128k, c4k, long
            brown
        } else if lower.contains("mamba")
            || lower.contains("rwkv")
            || lower.contains("hyena")
            || lower.contains("decoder")
        {
            // 🩵 架构类型: mamba, rwkv, hyena, decoder
            cyan
        } else {
            // 🔵 模型名称 (兜底)
            blue
        };

        tags.push((trimmed.to_string(), color));
    }

    tags
}

/// 参数量: 包含数字且以 b/m 结尾（k 留给上下文长度）
fn is_param_size(s: &str) -> bool {
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    has_digit && (s.ends_with('b') || s.ends_with('m'))
}

/// 量化类型: q4_k_m, q8_0, iq4_nl, iq2_xs … (排除 qwen/qwq 等模型名)
fn is_quantization(s: &str) -> bool {
    if s.starts_with("iq") && s.chars().nth(2).map_or(false, |c| c.is_ascii_digit()) {
        return true; // iq4_nl, iq2_xs …
    }
    // q 开头，第二个字符是数字（排除 "qwen", "qwq" 等模型名）
    s.starts_with('q') && s.chars().nth(1).map_or(false, |c| c.is_ascii_digit())
}

/// 训练方法关键词
fn is_training_method(s: &str) -> bool {
    s.contains("instruct")
        || s.contains("chat")
        || s.contains("sft")
        || s.contains("rlhf")
        || s.contains("dpo")
        || s.contains("orpo")
        || s.contains("grpo")
}

/// 上下文长度: 含 "128"/"64k"/"32k"/"c4k"/"long" 等模式
fn is_context_length(s: &str) -> bool {
    if s.ends_with('k') && s.contains(|c: char| c.is_ascii_digit()) {
        return true; // 128k, c4k, 4k, 32k...
    }
    s.contains("long") || s == "128" || s == "64" || s == "32"
}

/// 判断是否为 mmproj 文件
fn is_mmproj_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.contains("mmproj")
        || lower.contains("clip")
        || (lower.contains("proj") && lower.contains("vision"))
}

/// 判断是否为 DFlash 草稿文件
fn is_dflash_file(filename: &str) -> bool {
    filename.to_lowercase().contains("dflash")
}

/// 文件列表展示模式
#[derive(Clone, Copy, PartialEq)]
enum FileMode {
    Main,
    Mmproj,
    Dflash,
}

fn render_file_list(
    ui: &mut egui::Ui,
    dir: &std::path::Path,
    selected_path: std::path::PathBuf,
    on_select: &mut impl FnMut(std::path::PathBuf),
    lang: &i18n::Language,
    mode: FileMode,
) {
    let entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string().to_lowercase();
                if !name.ends_with(".gguf") {
                    return false;
                }
                match mode {
                    FileMode::Main => !is_mmproj_file(&name) && !is_dflash_file(&name),
                    FileMode::Mmproj => is_mmproj_file(&name),
                    FileMode::Dflash => is_dflash_file(&name),
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    if entries.is_empty() {
        ui.colored_label(
            egui::Color32::GRAY,
            match mode {
                FileMode::Main => i18n::t(i18n::Key::NoGgufFiles, lang),
                FileMode::Mmproj => i18n::t(i18n::Key::NoMmprojFiles, lang),
                FileMode::Dflash => i18n::t(i18n::Key::NoDflashFiles, lang),
            },
        );
        return;
    }

    for entry in entries {
        let file_path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let selected = selected_path == file_path;

        ui.horizontal(|ui| {
            // 标签
            let tags = parse_tags(&filename);
            for (text, color) in &tags {
                ui.add(
                    egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
                        .fill(*color)
                        .corner_radius(4.0),
                );
            }

            ui.separator();

            // 单选框
            if ui.add(egui::RadioButton::new(selected, "")).clicked() {
                on_select(file_path);
            }
        });
    }
}

pub fn ui(ui: &mut egui::Ui, settings: &mut AppSettings, lang: &i18n::Language) {
    ui.heading(i18n::t(i18n::Key::PanelModelTitle, lang));
    ui.separator();

    // 文件夹选择
    ui.horizontal(|ui| {
        ui.label(i18n::t(i18n::Key::LabelModelDir, lang));
        let mut dir_str = settings.model_dir.to_string_lossy().to_string();
        let response = ui.text_edit_singleline(&mut dir_str);
        if response.changed() {
            settings.model_dir = std::path::PathBuf::from(&dir_str);
        }
    });

    // 按钮行
    ui.horizontal(|ui| {
        if ui
            .button(i18n::t(i18n::Key::BtnSelectFolder, lang))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(i18n::t(i18n::Key::DialogSelectFolder, lang))
                .pick_folder()
            {
                settings.model_dir = path;
            }
        }
        if ui.button(i18n::t(i18n::Key::BtnAutoDetect, lang)).clicked() {
            if let Some(path) = auto_detect_model_dir() {
                settings.model_dir = path;
            } else {
                settings.model_dir = std::path::PathBuf::from("");
            }
        }
    });

    ui.add_space(12.0);

    // 文件夹为空时提示
    if settings.model_dir.as_os_str().is_empty() {
        ui.colored_label(egui::Color32::GRAY, i18n::t(i18n::Key::NoModelDir, lang));
        return;
    }

    // 模型文件列表
    ui.heading(i18n::t(i18n::Key::SectionModels, lang));
    ui.separator();
    let selected_model = settings.model_path.clone();
    render_file_list(
        ui,
        &settings.model_dir,
        selected_model,
        &mut |path| {
            settings.model_path = path;
        },
        lang,
        FileMode::Main,
    );

    // 分隔
    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionMmproj, lang));
    ui.separator();
    let selected_mmproj = settings.mmproj_path.clone();
    render_file_list(
        ui,
        &settings.model_dir,
        selected_mmproj.clone(),
        &mut |path| {
            // 再次点击已选中的路径 → 取消选中
            settings.mmproj_path = if selected_mmproj == path {
                std::path::PathBuf::new()
            } else {
                path
            };
        },
        lang,
        FileMode::Mmproj,
    );

    // DFlash 草稿文件
    ui.add_space(12.0);
    ui.heading(i18n::t(i18n::Key::SectionDflash, lang));
    ui.separator();
    let selected_dflash = settings.dflash_path.clone();
    render_file_list(
        ui,
        &settings.model_dir,
        selected_dflash.clone(),
        &mut |path| {
            // 再次点击已选中的路径 → 取消选中
            settings.dflash_path = if selected_dflash == path {
                std::path::PathBuf::new()
            } else {
                path
            };
        },
        lang,
        FileMode::Dflash,
    );
}
