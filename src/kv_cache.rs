use crate::config::settings::AppSettings;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::Path;

/// GGUF 模型信息摘要
#[derive(Debug)]
pub struct GgufInfo {
    /// 层数 (Block Count / llm.num_hidden_layers)
    pub block_count: usize,
    /// KV头数 (attention.key_head_count)
    pub kv_head_count: usize,
    /// 头维度 (head_dim.k / attention.head_dim)
    pub head_dim: usize,
    /// Embedding 维度 ({arch}.embedding_length)
    pub embedding_length: usize,
    /// 模型文件大小（字节）
    pub file_size: u64,
}

/// 从 GGUF 文件中读取模型信息
pub fn read_gguf_info(file_path: &Path) -> Result<GgufInfo, String> {
    // 获取文件 size
    let file_size = fs::metadata(file_path)
        .map(|m| m.len())
        .map_err(|e| format!("无法读取模型文件元数据: {}", e))?;

    // 打开 GGUF 容器
    let file_str = file_path
        .to_str()
        .ok_or_else(|| "模型文件路径包含无效字符".to_string())?;

    let mut container =
        gguf_rs::get_gguf_container(file_str).map_err(|e| format!("无法打开 GGUF 文件: {}", e))?;

    // 解码元数据（使用较大 max_array_size 以读取完整 token list）
    let model = container
        .decode()
        .map_err(|e| format!("GGUF 解码失败: {}", e))?;

    let kv = model.metadata();

    // 读取架构名称
    let arch = kv
        .get("general.architecture")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "无法从 GGUF 文件中读取架构信息".to_string())?;

    // 读取 block_count (层数)
    let block_key = format!("{}.block_count", arch);
    let block_count =
        kv.get(&block_key)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("无法从 GGUF 文件中读取块数 ({})", block_key))? as usize;

    // 读取 KV head count (fallback: attention.head_count)
    let kv_head_key = format!("{}.attention.key_head_count", arch);
    let kv_head_fallback = format!("{}.attention.head_count", arch);
    let kv_head_count = kv
        .get(&kv_head_key)
        .or_else(|| kv.get(&kv_head_fallback))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("无法从 GGUF 文件中读取 KV 头数 (尝试了 {} / {})", kv_head_key, kv_head_fallback))?
        as usize;

    // 读取 head dim (fallback: attention.key_length)
    let head_dim_key = format!("{}.head_dim", arch);
    let head_dim_fallback = format!("{}.attention.key_length", arch);
    let head_dim = kv
        .get(&head_dim_key)
        .or_else(|| kv.get(&head_dim_fallback))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("无法从 GGUF 文件中读取头维度 (尝试了 {} / {})", head_dim_key, head_dim_fallback))?
        as usize;

    // 读取 embedding length
    let emb_key = format!("{}.embedding_length", arch);
    let embedding_length = kv
        .get(&emb_key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("无法从 GGUF 文件中读取 Embedding 维度 ({})", emb_key))?
        as usize;

    Ok(GgufInfo {
        block_count,
        kv_head_count,
        head_dim,
        embedding_length,
        file_size,
    })
}

/// 执行 llama-server --list-devices 并解析空闲显存（MiB）
pub fn get_free_gpu_mib(server_path: &Path) -> Result<u64, String> {
    let exe = Path::new(&server_path);
    if !exe.exists() {
        return Err(format!("llama-server 不存在: {:?}", server_path));
    }

    // 使用 llama-server 所在目录的同名可执行文件来查询设备
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--list-devices")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x0800_0000u32); // CREATE_NO_WINDOW

    let output = cmd
        .output()
        .map_err(|e| format!("执行 --list-devices 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 正则匹配 "N MiB free" 模式，提取数字
    // 示例输出: (24560 MiB, 24410 MiB free) GPU0
    for line in stdout.lines() {
        if let Some(m) = regex_search_free_mib(line) {
            return Ok(m);
        }
    }

    // 尝试从 stderr 中查找（某些版本可能输出到 stderr）
    for line in stderr.lines() {
        if let Some(m) = regex_search_free_mib(line) {
            return Ok(m);
        }
    }

    Err(format!(
        "无法解析 --list-devices 输出中的空闲显存:\n{}",
        stdout
    ))
}

/// 在行中搜索 "N MiB free" 模式，返回 N（MiB）
fn regex_search_free_mib(line: &str) -> Option<u64> {
    // 手动匹配 "{number} MiB free" 子串
    let lower = line.to_lowercase();
    if !lower.contains("mib free") && !lower.contains(" mib free") {
        return None;
    }

    // 找到 "MiB free" 位置，向前读取数字
    for (i, c) in lower.char_indices() {
        if c == 'm' && lower[i..].starts_with("mib free") {
            // 从 m 之前开始向前查找空格分隔的数字
            let prefix = &line[..i];
            // trim trailing whitespace and parse the last number
            let trimmed = prefix.trim_end();
            if let Some(last_space) = trimmed.rfind(' ') {
                let num_str = &trimmed[last_space + 1..];
                if let Ok(n) = num_str.parse::<u64>() {
                    return Some(n);
                }
            } else {
                // 没有前导空格，整行就是数字（不太可能）
                if let Ok(n) = trimmed.parse::<u64>() {
                    return Some(n);
                }
            }
        }
    }

    None
}

/// KV 缓存类型 → 精度字节数（f64 以支持量化类型的非整数字节）
fn cache_type_precision_bytes(cache_type: &str) -> f64 {
    match cache_type {
        "f32" => 4.0,
        "f16" | "bf16" => 2.0,
        "q8_0" => 1.0,
        "q5_0" | "q5_1" => 0.625, // 5 bits per element
        "q4_0" | "q4_1" | "iq4_nl" => 0.5, // 4 bits per element
        _ => 2.0, // 默认 f16
    }
}

/// 计算最大可用上下文（k 为单位）
///
/// 公式：
///   Compute Buffer = parallel_slots × block_count × embedding_length × batch_size_actual × 4
///   单 token KV 占用 = kv_head_count × head_dim × (precision_k + precision_v) × block_count
///   最大 token 数 = ((GPU 空闲显存 - 模型文件) - Compute Buffer) / 单 token KV 占用 × kv_cache_ratio
///   返回值 = 最大 token 数 / 1024 （单位 k）
pub fn calc_max_context(gguf: &GgufInfo, settings: &AppSettings, free_mib: u64) -> u64 {
    // Compute Buffer（字节）= parallel_slots * block_count * embedding_length * batch_size_actual × 4
    let compute_buffer_bytes = (settings.parallel_slots as u64)
        .saturating_mul(gguf.block_count as u64)
        .saturating_mul(gguf.embedding_length as u64)
        .saturating_mul(settings.batch_size_actual() as u64)
        .saturating_mul(4);

    // 模型文件占用（MiB）
    let model_mib = gguf.file_size / (1024 * 1024);

    // GPU 空闲显存扣除模型文件后的可用空间（字节）
    let usable_bytes = (free_mib.saturating_sub(model_mib) as u64)
        .saturating_mul(1024)
        .saturating_mul(1024);

    // 扣掉 Compute Buffer 后剩余给 KV 缓存的空间（字节）
    let kv_bytes = usable_bytes.saturating_sub(compute_buffer_bytes);

    if kv_bytes == 0 {
        return 0;
    }

    // K 和 V 的精度字节数
    let precision_k = cache_type_precision_bytes(&settings.cache_type_k);
    let precision_v = cache_type_precision_bytes(&settings.cache_type_v);

    // 单个 token 的 KV 缓存占用（字节）= kv_head_count × head_dim × (precision_k + precision_v) × block_count
    let per_token_kv_bytes = gguf.kv_head_count as f64
        * gguf.head_dim as f64
        * (precision_k + precision_v)
        * gguf.block_count as f64;

    if per_token_kv_bytes <= 0.0 {
        return 0;
    }

    // 最大 token 数 = 剩余字节 / 单 token 占用 × kv_cache_ratio
    let max_tokens = (kv_bytes as f64) / per_token_kv_bytes * settings.kv_cache_ratio as f64;

    // 返回 k 单位（1k = 1024）
    (max_tokens / 1024.0) as u64
}

/// 计算 KV 缓存可用空间
/// 公式: (GPU空闲显存 - 模型文件大小) - (并发数量 × block_count × embedding_length × 最大物理批次大小 × 2字节/f16 × 2缓冲倍率)
pub fn calc_kv_cache_space(gguf: &GgufInfo, settings: &AppSettings, free_mib: u64) -> String {
    // Compute Buffer（字节）= parallel_slots * block_count * embedding_length * batch_size_actual * 2(f16) * 2(缓冲倍率)
    let compute_buffer_bytes = (settings.parallel_slots as u64)
        .saturating_mul(gguf.block_count as u64)
        .saturating_mul(gguf.embedding_length as u64)
        .saturating_mul(settings.batch_size_actual() as u64)
        .saturating_mul(4);

    // 模型文件占用（MiB）
    let model_mib = gguf.file_size / (1024 * 1024);

    // GPU空闲显存扣除模型文件后的可用空间（MiB）
    let usable_mib = free_mib.saturating_sub(model_mib);

    // Compute Buffer（MiB）
    let compute_buffer_mib = compute_buffer_bytes / (1024 * 1024);

    if compute_buffer_mib > usable_mib {
        format!("{} MB / {} MB（超出 {} MB）", usable_mib, compute_buffer_mib, compute_buffer_mib - usable_mib)
    } else {
        let remaining = usable_mib - compute_buffer_mib;
        format!("{} MB / {} MB（剩余 {} MB）", compute_buffer_mib, usable_mib, remaining)
    }
}

/// Facade function：聚合读取 GGUF + GPU 信息 → 计算并格式化结果
pub fn calc_and_format(settings: &AppSettings) -> Result<String, String> {
    log::info!("[calc_and_format] 开始计算 KV 缓存空间");
    log::info!("[calc_and_format] model_path = {:?}", settings.model_path);
    log::info!("[calc_and_format] server_path = {:?}", settings.server_path);

    // 1. 读取 GGUF 模型信息
    let gguf = read_gguf_info(&settings.model_path)?;
    log::info!("[calc_and_format] GGUF info: block_count={}, kv_head_count={}, head_dim={}, embedding_length={}, file_size={} bytes",
        gguf.block_count, gguf.kv_head_count, gguf.head_dim, gguf.embedding_length, gguf.file_size);

    // 2. 获取空闲显存
    let free_mib = get_free_gpu_mib(&settings.server_path)?;
    log::info!("[calc_and_format] GPU 空闲显存: {} MiB", free_mib);

    // 3. 计算并格式化
    let result = calc_kv_cache_space(&gguf, settings, free_mib);
    log::info!("[calc_and_format] KV 缓存计算结果: {}", result);
    Ok(result)
}

/// Facade function：聚合读取 GGUF + GPU 信息 → 返回最大上下文（k 单位）
pub fn calc_max_context_facade(settings: &AppSettings) -> Result<usize, String> {
    log::info!("[calc_max_context_facade] 开始计算最大上下文");

    // 1. 读取 GGUF 模型信息
    let gguf = read_gguf_info(&settings.model_path)?;
    log::info!("[calc_max_context_facade] GGUF info: block_count={}, kv_head_count={}, head_dim={}, embedding_length={}, file_size={} bytes",
        gguf.block_count, gguf.kv_head_count, gguf.head_dim, gguf.embedding_length, gguf.file_size);

    // 2. 获取空闲显存
    let free_mib = get_free_gpu_mib(&settings.server_path)?;
    log::info!("[calc_max_context_facade] GPU 空闲显存: {} MiB", free_mib);

    // 3. 计算最大上下文（k 单位）
    let max_ctx_k = calc_max_context(&gguf, settings, free_mib);
    log::info!("[calc_max_context_facade] cache_type_k={}, cache_type_v={}, kv_cache_ratio={}",
        settings.cache_type_k, settings.cache_type_v, settings.kv_cache_ratio);
    log::info!("[calc_max_context_facade] 最大可用上下文: {}k", max_ctx_k);
    Ok(max_ctx_k as usize)
}
