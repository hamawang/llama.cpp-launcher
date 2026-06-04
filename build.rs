fn main() {
    // build.rs 运行在 host 上，不能用 #[cfg(windows)] 判断目标平台
    // 必须通过 CARGO_CFG_TARGET_OS 环境变量判断
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("llama-blue.ico");
        res.set("ProductName", "llama.cpp launcher");
        res.set("FileDescription", "llama.cpp launcher - GUI Launcher");
        res.set("LegalCopyright", "Copyright 2025");
        res.set("InternalName", "llama.cpp launcher");
        res.set("OriginalFilename", "llama_cpp_launcher.exe");
        res.set_version_info(winres::VersionInfo::FILEVERSION, 0x0000000100000000u64);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0000000100000000u64);
        res.set_version_info(winres::VersionInfo::FILEOS, 0x40004u64);
        res.set_version_info(winres::VersionInfo::FILETYPE, 0x2u64);
        res.compile().unwrap();
    }
    // 非 Windows 平台不做任何操作
}
