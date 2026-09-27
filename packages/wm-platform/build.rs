fn main() {
  #[cfg(target_os = "macos")]
  println!(
    "cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks"
  );

  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
    shaders::compile();
  }
}

/// Compiles the HLSL shaders to bytecode at build time, so the WM has no
/// runtime dependency on the shader compiler.
mod shaders {
  use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
  };

  /// Every shader source, for rebuilds; the picture shaders include the
  /// theme's.
  const SOURCES: [&str; 3] = [
    "shaders/color_theme.hlsl",
    "shaders/color_theme_pictures.hlsl",
    "shaders/color_theme_keep.hlsl",
  ];

  /// `(source, entry point, profile, output file)`. Shader model 4.0 runs
  /// on every D3D11 device, down to feature level 10.0 and WARP; only
  /// `keep_pictures`' compute shader needs 5.0 (feature level 11.0).
  const STAGES: [(&str, &str, &str, &str); 6] = [
    (
      "shaders/color_theme.hlsl",
      "vs_main",
      "vs_4_0",
      "color_theme_vs.cso",
    ),
    (
      "shaders/color_theme.hlsl",
      "ps_main",
      "ps_4_0",
      "color_theme_ps.cso",
    ),
    (
      "shaders/color_theme.hlsl",
      "ps_sample",
      "ps_4_0",
      "color_theme_sample_ps.cso",
    ),
    (
      "shaders/color_theme_pictures.hlsl",
      "ps_blocks",
      "ps_4_0",
      "color_theme_blocks_ps.cso",
    ),
    (
      "shaders/color_theme_pictures.hlsl",
      "ps_pictures",
      "ps_4_0",
      "color_theme_pictures_ps.cso",
    ),
    (
      "shaders/color_theme_keep.hlsl",
      "cs_keep",
      "cs_5_0",
      "color_theme_keep_cs.cso",
    ),
  ];

  pub fn compile() {
    for source in SOURCES {
      println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-env-changed=FXC");

    let out_dir =
      PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let fxc = find_fxc().unwrap_or_else(|| {
      panic!(
        "fxc.exe not found. Install the Windows SDK, or point the `FXC` \
         environment variable at fxc.exe."
      )
    });

    for (source, entry, profile, output) in STAGES {
      let status = Command::new(&fxc)
        .args([
          "/nologo", "/O3", "/WX", "/Ges", "/T", profile, "/E", entry,
        ])
        .arg("/Fo")
        .arg(out_dir.join(output))
        .arg(source)
        .status()
        .unwrap_or_else(|err| {
          panic!("Failed to run {}: {err}", fxc.display())
        });

      assert!(
        status.success(),
        "fxc failed to compile {entry} in {source}"
      );
    }
  }

  /// Finds `fxc.exe` via the `FXC` override, else the newest Windows 10+
  /// SDK installed.
  fn find_fxc() -> Option<PathBuf> {
    if let Some(path) = env::var_os("FXC") {
      return Some(PathBuf::from(path));
    }

    // fxc runs on the build host, so this is the host's architecture.
    let arch = if cfg!(target_arch = "aarch64") {
      "arm64"
    } else if cfg!(target_arch = "x86") {
      "x86"
    } else {
      "x64"
    };

    let program_files = env::var_os("ProgramFiles(x86)")
      .or_else(|| env::var_os("ProgramFiles"))?;
    let bin_dir = Path::new(&program_files).join("Windows Kits/10/bin");

    // SDK directories are named by version, e.g. `10.0.26100.0`.
    std::fs::read_dir(&bin_dir)
      .ok()?
      .filter_map(Result::ok)
      .filter_map(|entry| {
        let version = entry
          .file_name()
          .to_str()?
          .split('.')
          .map(str::parse::<u32>)
          .collect::<Result<Vec<_>, _>>()
          .ok()?;
        let path = entry.path().join(arch).join("fxc.exe");
        path.is_file().then_some((version, path))
      })
      .max_by(|(a, _), (b, _)| a.cmp(b))
      .map(|(_, path)| path)
  }
}
