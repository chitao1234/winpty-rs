use std::env;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;

use which::which;

const LIBWINPTY_SOURCES: &[&str] = &[
    "libwinpty/AgentLocation.cc",
    "libwinpty/winpty.cc",
    "shared/BackgroundDesktop.cc",
    "shared/Buffer.cc",
    "shared/DebugClient.cc",
    "shared/GenRandom.cc",
    "shared/OwnedHandle.cc",
    "shared/StringUtil.cc",
    "shared/WindowsSecurity.cc",
    "shared/WindowsVersion.cc",
    "shared/WinptyAssert.cc",
    "shared/WinptyException.cc",
    "shared/WinptyVersion.cc",
];

const AGENT_SOURCES: &[&str] = &[
    "agent/Agent.cc",
    "agent/AgentCreateDesktop.cc",
    "agent/ConsoleFont.cc",
    "agent/ConsoleInput.cc",
    "agent/ConsoleInputReencoding.cc",
    "agent/ConsoleLine.cc",
    "agent/DebugShowInput.cc",
    "agent/DefaultInputMap.cc",
    "agent/EventLoop.cc",
    "agent/InputMap.cc",
    "agent/LargeConsoleRead.cc",
    "agent/NamedPipe.cc",
    "agent/Scraper.cc",
    "agent/Terminal.cc",
    "agent/Win32Console.cc",
    "agent/Win32ConsoleBuffer.cc",
    "agent/main.cc",
    "shared/BackgroundDesktop.cc",
    "shared/Buffer.cc",
    "shared/DebugClient.cc",
    "shared/GenRandom.cc",
    "shared/OwnedHandle.cc",
    "shared/StringUtil.cc",
    "shared/WindowsSecurity.cc",
    "shared/WindowsVersion.cc",
    "shared/WinptyAssert.cc",
    "shared/WinptyException.cc",
    "shared/WinptyVersion.cc",
];

const PUBLIC_HEADERS: &[&str] = &["winpty.h", "winpty_constants.h"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
struct TargetInfo {
    triple: String,
    arch: String,
    env: String,
    os: String,
}

impl TargetInfo {
    fn read() -> Self {
        Self {
            triple: env::var("TARGET").unwrap(),
            arch: env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default(),
            env: env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default(),
            os: env::var("CARGO_CFG_TARGET_OS").unwrap_or_default(),
        }
    }

    fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    fn is_gnu(&self) -> bool {
        self.env == "gnu"
    }

    fn is_msvc(&self) -> bool {
        self.env == "msvc"
    }
}

#[derive(Clone, Debug)]
struct LibrarySpec<'a> {
    package_name: &'a str,
    root_var: &'a str,
    lib_var: &'a str,
    bin_var: &'a str,
    include_var: &'a str,
    static_names: &'a [&'a str],
    import_names: &'a [&'a str],
    dll_names: &'a [&'a str],
    runtime_names: &'a [&'a str],
}

#[derive(Clone, Debug)]
struct LibraryLayout {
    root: PathBuf,
    lib_dir: PathBuf,
    bin_dir: PathBuf,
    include_dir: Option<PathBuf>,
    static_lib: Option<PathBuf>,
    import_lib: Option<PathBuf>,
    dll: Option<PathBuf>,
    runtime: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct VendoredPaths {
    source_root: PathBuf,
    src_root: PathBuf,
    include_root: PathBuf,
    build_root: PathBuf,
    gen_dir: PathBuf,
    shared_build_dir: PathBuf,
    static_build_dir: PathBuf,
    agent_build_dir: PathBuf,
    stage_root: PathBuf,
    stage_lib_dir: PathBuf,
    stage_static_lib_dir: PathBuf,
    stage_bin_dir: PathBuf,
    stage_include_dir: PathBuf,
}

impl VendoredPaths {
    fn new(source_root: PathBuf, out_dir: PathBuf) -> Self {
        let build_root = out_dir.join("vendored-winpty-build");
        let stage_root = out_dir.join("vendored-winpty-install");

        Self {
            src_root: source_root.join("src"),
            include_root: source_root.join("src").join("include"),
            gen_dir: build_root.join("gen"),
            shared_build_dir: build_root.join("shared-lib"),
            static_build_dir: build_root.join("static-lib"),
            agent_build_dir: build_root.join("agent"),
            stage_lib_dir: stage_root.join("lib"),
            stage_static_lib_dir: stage_root.join("static-lib"),
            stage_bin_dir: stage_root.join("bin"),
            stage_include_dir: stage_root.join("include").join("winpty"),
            source_root,
            build_root,
            stage_root,
        }
    }

    fn prepare(&self) -> Result<(), String> {
        recreate_dir(&self.build_root)?;
        recreate_dir(&self.stage_root)?;
        fs::create_dir_all(&self.gen_dir)
            .map_err(|err| format!("failed to create {}: {err}", self.gen_dir.display()))?;
        fs::create_dir_all(&self.stage_lib_dir)
            .map_err(|err| format!("failed to create {}: {err}", self.stage_lib_dir.display()))?;
        fs::create_dir_all(&self.stage_static_lib_dir).map_err(|err| {
            format!(
                "failed to create {}: {err}",
                self.stage_static_lib_dir.display()
            )
        })?;
        fs::create_dir_all(&self.stage_bin_dir)
            .map_err(|err| format!("failed to create {}: {err}", self.stage_bin_dir.display()))?;
        fs::create_dir_all(&self.stage_include_dir).map_err(|err| {
            format!(
                "failed to create {}: {err}",
                self.stage_include_dir.display()
            )
        })?;
        Ok(())
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/winpty");
    println!("cargo:rerun-if-env-changed=WINPTYRS_STAGE_RUNTIME");

    let host = env::var("HOST").unwrap();
    let target = TargetInfo::read();

    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    if !target.is_windows() {
        return;
    }

    let layout = if env::var_os("CARGO_FEATURE_VENDORED").is_some() {
        build_vendored_layout(&target).unwrap_or_else(|err| panic!("{err}"))
    } else {
        find_winpty_layout(&host)
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| {
                panic!(
                    "winpty not found for target {}; set WINPTY_DIR, WINPTY_LIB_DIR, WINPTY_BIN_DIR, or enable the vendored feature",
                    target.triple
                )
            })
    };

    let link_kind = configure_winpty(&target, &layout).unwrap_or_else(|err| panic!("{err}"));
    stage_runtime_files(&target, &layout, link_kind).unwrap_or_else(|err| panic!("{err}"));
}

fn build_vendored_layout(target: &TargetInfo) -> Result<LibraryLayout, String> {
    let source_root = env_path("WINPTY_SOURCE_DIR")
        .or_else(default_vendor_source)
        .ok_or_else(|| {
            "vendored feature requires bundled vendor/winpty sources or WINPTY_SOURCE_DIR"
                .to_string()
        })?;
    println!("cargo:rerun-if-changed={}", source_root.display());

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let paths = VendoredPaths::new(source_root, out_dir);
    paths.prepare()?;

    generate_version_header(&paths)?;
    stage_public_headers(&paths)?;

    let (import_lib, dll) = build_vendored_dynamic_lib(target, &paths)?;
    let static_lib = build_vendored_static_lib(target, &paths)?;
    let runtime = build_vendored_agent(target, &paths)?;

    Ok(LibraryLayout {
        root: paths.stage_root.clone(),
        lib_dir: paths.stage_lib_dir.clone(),
        bin_dir: paths.stage_bin_dir.clone(),
        include_dir: Some(paths.stage_include_dir),
        static_lib: Some(static_lib),
        import_lib: Some(import_lib),
        dll: Some(dll),
        runtime: Some(runtime),
    })
}

fn default_vendor_source() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR")?);
    let candidate = manifest_dir.join("vendor").join("winpty");
    candidate.is_dir().then_some(candidate)
}

fn recreate_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
    }
    fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {err}", path.display()))
}

fn generate_version_header(paths: &VendoredPaths) -> Result<(), String> {
    let version_path = paths.source_root.join("VERSION.txt");
    let version = fs::read_to_string(&version_path)
        .map_err(|err| format!("failed to read {}: {err}", version_path.display()))?
        .trim_end_matches(['\r', '\n'])
        .to_string();
    let commit = env_var("WINPTY_COMMIT_HASH")
        .map(|value| value.to_string_lossy().trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_string());

    let header = format!(
        "// AUTO-GENERATED BY build.rs\nconst char GenVersion_Version[] = {version:?};\nconst char GenVersion_Commit[] = {commit:?};\n"
    );
    let output_path = paths.gen_dir.join("GenVersion.h");
    fs::write(&output_path, header)
        .map_err(|err| format!("failed to write {}: {err}", output_path.display()))
}

fn stage_public_headers(paths: &VendoredPaths) -> Result<(), String> {
    for header in PUBLIC_HEADERS {
        let source = paths.include_root.join(header);
        let dest = paths.stage_include_dir.join(header);
        copy_file(&source, &dest)?;
    }
    Ok(())
}

fn build_vendored_dynamic_lib(
    target: &TargetInfo,
    paths: &VendoredPaths,
) -> Result<(PathBuf, PathBuf), String> {
    fs::create_dir_all(&paths.shared_build_dir).map_err(|err| {
        format!(
            "failed to create {}: {err}",
            paths.shared_build_dir.display()
        )
    })?;

    let mut build = common_cpp_build(paths, &paths.shared_build_dir);
    build.define("COMPILING_WINPTY_DLL", None);
    for source in LIBWINPTY_SOURCES {
        build.file(paths.src_root.join(source));
    }

    let objects = build
        .try_compile_intermediates()
        .map_err(|err| format!("failed to compile vendored winpty.dll objects: {err}"))?;
    let compiler = build
        .try_get_compiler()
        .map_err(|err| format!("failed to resolve the C++ compiler for winpty.dll: {err}"))?;

    let dll = paths.stage_bin_dir.join("winpty.dll");
    let import_lib = if target.is_msvc() {
        paths.stage_lib_dir.join("winpty.lib")
    } else {
        paths.stage_lib_dir.join("libwinpty.dll.a")
    };

    link_shared_library(target, &compiler, &objects, &dll, &import_lib)?;
    Ok((import_lib, dll))
}

fn build_vendored_static_lib(
    target: &TargetInfo,
    paths: &VendoredPaths,
) -> Result<PathBuf, String> {
    fs::create_dir_all(&paths.static_build_dir).map_err(|err| {
        format!(
            "failed to create {}: {err}",
            paths.static_build_dir.display()
        )
    })?;

    let mut build = common_cpp_build(paths, &paths.static_build_dir);
    build.define("WINPTY_STATIC", None);
    for source in LIBWINPTY_SOURCES {
        build.file(paths.src_root.join(source));
    }

    build
        .try_compile("winpty")
        .map_err(|err| format!("failed to compile vendored libwinpty: {err}"))?;

    let built_library = find_existing(
        std::slice::from_ref(&paths.static_build_dir),
        if target.is_msvc() {
            &["winpty.lib", "libwinpty.a"]
        } else {
            &["libwinpty.a", "winpty.lib"]
        },
    )
    .ok_or_else(|| {
        format!(
            "cc build did not produce a usable static winpty library in {}",
            paths.static_build_dir.display()
        )
    })?;

    let staged_library = if target.is_msvc() {
        paths.stage_static_lib_dir.join("winpty.lib")
    } else {
        paths.stage_static_lib_dir.join("libwinpty.a")
    };
    copy_file(&built_library, &staged_library)?;
    Ok(staged_library)
}

fn build_vendored_agent(target: &TargetInfo, paths: &VendoredPaths) -> Result<PathBuf, String> {
    fs::create_dir_all(&paths.agent_build_dir).map_err(|err| {
        format!(
            "failed to create {}: {err}",
            paths.agent_build_dir.display()
        )
    })?;

    let mut build = common_cpp_build(paths, &paths.agent_build_dir);
    build.define("WINPTY_AGENT_ASSERT", None);
    for source in AGENT_SOURCES {
        build.file(paths.src_root.join(source));
    }

    let objects = build
        .try_compile_intermediates()
        .map_err(|err| format!("failed to compile vendored winpty agent objects: {err}"))?;
    let compiler = build
        .try_get_compiler()
        .map_err(|err| format!("failed to resolve the C++ compiler for winpty-agent: {err}"))?;

    let output = paths.stage_bin_dir.join("winpty-agent.exe");
    link_agent_executable(target, &compiler, &objects, &output)?;
    Ok(output)
}

fn common_cpp_build(paths: &VendoredPaths, out_dir: &Path) -> cc::Build {
    let mut build = cc::Build::new();
    build.cpp(true);
    build.cargo_metadata(false);
    build.out_dir(out_dir);
    build.include(&paths.src_root);
    build.include(&paths.include_root);
    build.include(&paths.gen_dir);
    build.define("UNICODE", None);
    build.define("_UNICODE", None);
    build.define("_WIN32_WINNT", Some("0x0501"));
    build.define("NOMINMAX", None);
    build.flag_if_supported("-std=c++11");
    build.flag_if_supported("/EHsc");
    build
}

fn link_agent_executable(
    target: &TargetInfo,
    compiler: &cc::Tool,
    objects: &[PathBuf],
    output: &Path,
) -> Result<(), String> {
    let mut command = compiler.to_command();

    if compiler.is_like_msvc() {
        command.args(objects);
        command.arg("/link");
        command.arg(format!("/OUT:{}", output.display()));
        command.arg("advapi32.lib");
        command.arg("shell32.lib");
        command.arg("user32.lib");
    } else {
        command.arg("-o");
        command.arg(output);
        if target.is_gnu() {
            command.arg("-static");
            command.arg("-static-libgcc");
            command.arg("-static-libstdc++");
        }
        command.args(objects);
        command.arg("-ladvapi32");
        command.arg("-lshell32");
        command.arg("-luser32");
    }

    run(&mut command)
}

fn link_shared_library(
    target: &TargetInfo,
    compiler: &cc::Tool,
    objects: &[PathBuf],
    dll: &Path,
    import_lib: &Path,
) -> Result<(), String> {
    let mut command = compiler.to_command();

    if compiler.is_like_msvc() {
        command.args(objects);
        command.arg("/link");
        command.arg("/DLL");
        command.arg(format!("/OUT:{}", dll.display()));
        command.arg(format!("/IMPLIB:{}", import_lib.display()));
        command.arg("advapi32.lib");
        command.arg("user32.lib");
    } else {
        command.arg("-shared");
        command.arg("-o");
        command.arg(dll);
        if target.is_gnu() {
            command.arg("-static");
            command.arg("-static-libgcc");
            command.arg("-static-libstdc++");
        }
        command.args(objects);
        command.arg(format!("-Wl,--out-implib,{}", import_lib.display()));
        command.arg("-ladvapi32");
        command.arg("-luser32");
    }

    run(&mut command)
}

fn copy_file(source: &Path, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::copy(source, dest).map_err(|err| {
        format!(
            "failed to copy {} to {}: {err}",
            source.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn run(command: &mut Command) -> Result<(), String> {
    let rendered = format!("{command:?}");
    let output = command
        .output()
        .map_err(|err| format!("failed to run {rendered}: {err}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut message = format!("{rendered} exited with status {}", output.status);
    if !stdout.is_empty() {
        message.push_str(&format!("\nstdout:\n{stdout}"));
    }
    if !stderr.is_empty() {
        message.push_str(&format!("\nstderr:\n{stderr}"));
    }
    Err(message)
}

fn configure_winpty(target: &TargetInfo, layout: &LibraryLayout) -> Result<LinkKind, String> {
    let agent = layout
        .runtime
        .as_ref()
        .ok_or_else(|| "winpty layout is missing winpty-agent.exe".to_string())?;

    validate_pe_arch(agent, &target.arch, "winpty-agent.exe")?;
    if let Some(dll) = &layout.dll {
        validate_pe_arch(dll, &target.arch, "winpty.dll")?;
    }

    let prefer_static = env_bool("WINPTY_STATIC");
    let can_dynamic = layout.import_lib.is_some();
    let can_static = layout.static_lib.is_some();

    let link_kind = match prefer_static {
        Some(true) => {
            if can_static {
                LinkKind::Static
            } else {
                return Err(
                    "WINPTY_STATIC=1 was set, but no static winpty library was found".into(),
                );
            }
        }
        Some(false) => {
            if can_dynamic {
                LinkKind::Dynamic
            } else {
                return Err(
                    "WINPTY_STATIC=0 was set, but no dynamic winpty import library was found"
                        .into(),
                );
            }
        }
        None => {
            if can_dynamic {
                LinkKind::Dynamic
            } else if can_static {
                LinkKind::Static
            } else {
                return Err("winpty layout did not provide a static or dynamic library".into());
            }
        }
    };

    emit_common_metadata("winpty", layout);
    println!(
        "cargo:link_kind={}",
        match link_kind {
            LinkKind::Static => "static",
            LinkKind::Dynamic => "dynamic",
        }
    );

    match link_kind {
        LinkKind::Dynamic => {
            let import_lib = layout.import_lib.as_ref().unwrap();
            let link_dir = emit_dynamic_link("winpty", target, import_lib)?;
            println!("cargo:rustc-link-search=native={}", link_dir.display());
            println!("cargo:rustc-link-lib=dylib=winpty");
        }
        LinkKind::Static => {
            let static_lib = layout.static_lib.as_ref().unwrap();
            let link_dir = static_lib.parent().ok_or_else(|| {
                "static winpty library does not have a parent directory".to_string()
            })?;
            println!("cargo:rustc-link-search=native={}", link_dir.display());
            println!("cargo:rustc-link-lib=static=winpty");
            emit_winpty_static_dependencies(target);
        }
    }

    println!("cargo:rustc-cfg=winpty_available");
    Ok(link_kind)
}

fn emit_common_metadata(prefix: &str, layout: &LibraryLayout) {
    println!("cargo:{prefix}_root={}", layout.root.display());
    println!("cargo:{prefix}_lib_dir={}", layout.lib_dir.display());
    println!("cargo:{prefix}_bin_dir={}", layout.bin_dir.display());

    if let Some(include_dir) = &layout.include_dir {
        println!("cargo:{prefix}_include={}", include_dir.display());
    }
    if let Some(dll) = &layout.dll {
        println!("cargo:{prefix}_dll={}", dll.display());
    }
    if let Some(runtime) = &layout.runtime {
        println!("cargo:{prefix}_runtime={}", runtime.display());
    }
    if let Some(import_lib) = &layout.import_lib {
        println!("cargo:{prefix}_import_lib={}", import_lib.display());
    }
    if let Some(static_lib) = &layout.static_lib {
        println!("cargo:{prefix}_static_lib={}", static_lib.display());
    }
}

fn stage_runtime_files(
    target: &TargetInfo,
    layout: &LibraryLayout,
    link_kind: LinkKind,
) -> Result<(), String> {
    if !target.is_windows() || env::var("WINPTYRS_STAGE_RUNTIME").ok().as_deref() != Some("1") {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let profile_dir = profile_dir(&out_dir).ok_or_else(|| {
        format!(
            "OUT_DIR was not nested under target/<triple>/<profile>/build: {}",
            out_dir.display()
        )
    })?;

    let runtime = layout
        .runtime
        .as_ref()
        .ok_or_else(|| "winpty layout is missing winpty-agent.exe".to_string())?;
    copy_into_dir(runtime, &profile_dir)?;
    copy_into_dir(runtime, &profile_dir.join("deps"))?;

    if link_kind == LinkKind::Dynamic {
        let dll = layout
            .dll
            .as_ref()
            .ok_or_else(|| "dynamic winpty layout is missing winpty.dll".to_string())?;
        copy_into_dir(dll, &profile_dir)?;
        copy_into_dir(dll, &profile_dir.join("deps"))?;
    }

    Ok(())
}

fn profile_dir(out_dir: &Path) -> Option<PathBuf> {
    out_dir.ancestors().nth(3).map(Path::to_path_buf)
}

fn copy_into_dir(source: &Path, dest_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dest_dir)
        .map_err(|err| format!("failed to create {}: {err}", dest_dir.display()))?;

    let dest = dest_dir.join(
        source
            .file_name()
            .ok_or_else(|| format!("runtime artifact has no file name: {}", source.display()))?,
    );

    fs::copy(source, &dest).map_err(|err| {
        format!(
            "failed to copy runtime artifact {} to {}: {err}",
            source.display(),
            dest.display()
        )
    })?;

    Ok(())
}

fn emit_dynamic_link(
    library_name: &str,
    target: &TargetInfo,
    import_lib: &Path,
) -> Result<PathBuf, String> {
    if !target.is_gnu() {
        return import_lib.parent().map(Path::to_path_buf).ok_or_else(|| {
            format!("{library_name} import library does not have a parent directory")
        });
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let staged_dir = out_dir.join(format!("{library_name}-gnu-import"));
    fs::create_dir_all(&staged_dir)
        .map_err(|err| format!("failed to create {}: {err}", staged_dir.display()))?;

    let staged_import = staged_dir.join(format!("lib{library_name}.dll.a"));
    fs::copy(import_lib, &staged_import).map_err(|err| {
        format!(
            "failed to stage {} as {}: {err}",
            import_lib.display(),
            staged_import.display()
        )
    })?;

    Ok(staged_dir)
}

fn emit_winpty_static_dependencies(target: &TargetInfo) {
    if target.is_gnu() {
        emit_gnu_static_runtime_search_paths(target)
            .unwrap_or_else(|err| panic!("failed to locate MinGW static runtime libraries: {err}"));
        println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=static=gcc_eh");
        println!("cargo:rustc-link-lib=static=gcc");
        println!("cargo:rustc-link-lib=static=winpthread");
    }

    println!("cargo:rustc-link-lib=dylib=advapi32");
    println!("cargo:rustc-link-lib=dylib=user32");
}

fn emit_gnu_static_runtime_search_paths(target: &TargetInfo) -> Result<(), String> {
    let cxx = resolve_gnu_tool(target, "CXX", "g++")?;
    let cc = resolve_gnu_tool(target, "CC", "gcc")?;

    let search_paths = unique_dirs(&[
        tool_print_file_name(&cxx, "libstdc++.a")?,
        tool_print_file_name(&cc, "libgcc_eh.a")?,
        tool_print_file_name(&cc, "libgcc.a")?,
        tool_print_file_name(&cc, "libwinpthread.a")?,
    ]);

    for path in search_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    Ok(())
}

fn resolve_gnu_tool(target: &TargetInfo, env_name: &str, suffix: &str) -> Result<String, String> {
    if let Some(explicit) = env_var(env_name) {
        let explicit = explicit.to_string_lossy().trim().to_string();
        if !explicit.is_empty() {
            return Ok(explicit);
        }
    }

    let mut candidates = vec![format!("{}-{suffix}", target.triple)];

    if let Some(prefix) = mingw_tool_prefix(target) {
        candidates.push(format!("{prefix}-{suffix}"));
    }

    for candidate in &candidates {
        if which(candidate).is_ok() {
            return Ok(candidate.clone());
        }
    }

    Err(format!(
        "no usable {suffix} tool found for target {}; tried {}",
        target.triple,
        candidates.join(", ")
    ))
}

fn mingw_tool_prefix(target: &TargetInfo) -> Option<&'static str> {
    match target.arch.as_str() {
        "x86_64" => Some("x86_64-w64-mingw32"),
        "x86" => Some("i686-w64-mingw32"),
        "aarch64" => Some("aarch64-w64-mingw32"),
        _ => None,
    }
}

fn tool_print_file_name(tool: &str, file_name: &str) -> Result<PathBuf, String> {
    let output = Command::new(tool)
        .arg(format!("-print-file-name={file_name}"))
        .output()
        .map_err(|err| format!("failed to run {tool} for {file_name}: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "{tool} -print-file-name={file_name} exited with status {}",
            output.status
        ));
    }

    let printed = String::from_utf8(output.stdout)
        .map_err(|err| format!("failed to decode {tool} output for {file_name}: {err}"))?;
    let library_path = PathBuf::from(printed.trim());

    if !library_path.is_file() {
        return Err(format!(
            "{tool} reported {}, but the file does not exist",
            library_path.display()
        ));
    }

    library_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        format!(
            "{tool} reported {}, but it has no parent directory",
            library_path.display()
        )
    })
}

fn find_winpty_layout(host: &str) -> Result<Option<LibraryLayout>, String> {
    let fallback_root = if host.contains("windows") {
        auto_winpty_root()
    } else {
        None
    };

    resolve_layout(
        &LibrarySpec {
            package_name: "winpty",
            root_var: "WINPTY_DIR",
            lib_var: "WINPTY_LIB_DIR",
            bin_var: "WINPTY_BIN_DIR",
            include_var: "WINPTY_INCLUDE_DIR",
            static_names: &["libwinpty.a"],
            import_names: &["winpty.lib", "libwinpty.dll.a"],
            dll_names: &["winpty.dll"],
            runtime_names: &["winpty-agent.exe", "winpty-agent"],
        },
        fallback_root,
    )
}

fn resolve_layout(
    spec: &LibrarySpec<'_>,
    fallback_root: Option<PathBuf>,
) -> Result<Option<LibraryLayout>, String> {
    let root_override = env_path(spec.root_var);
    let lib_override = env_path(spec.lib_var);
    let bin_override = env_path(spec.bin_var);
    let include_override = env_path(spec.include_var);
    let explicit = root_override.is_some()
        || lib_override.is_some()
        || bin_override.is_some()
        || include_override.is_some();

    let root = match root_override {
        Some(root) => root,
        None => {
            if let Some(root) = common_parent(lib_override.as_deref(), bin_override.as_deref()) {
                root
            } else if let Some(fallback_root) = fallback_root {
                fallback_root
            } else if let Some(path) = lib_override.clone().or_else(|| bin_override.clone()) {
                path
            } else {
                return Ok(None);
            }
        }
    };

    let lib_dir = lib_override.unwrap_or_else(|| pick_layout_dir(&root, "lib"));
    let bin_dir = bin_override.unwrap_or_else(|| pick_layout_dir(&root, "bin"));
    let include_dir = include_override.or_else(|| {
        let candidate = root.join("include");
        candidate.is_dir().then_some(candidate)
    });

    let search_dirs = unique_dirs(&[root.clone(), lib_dir.clone(), bin_dir.clone()]);

    let layout = LibraryLayout {
        static_lib: find_existing(&search_dirs, spec.static_names),
        import_lib: find_existing(&search_dirs, spec.import_names),
        dll: find_existing(&search_dirs, spec.dll_names),
        runtime: find_existing(&search_dirs, spec.runtime_names),
        root,
        lib_dir,
        bin_dir,
        include_dir,
    };

    let found_any_artifact = layout.static_lib.is_some()
        || layout.import_lib.is_some()
        || layout.dll.is_some()
        || layout.runtime.is_some();

    if found_any_artifact {
        Ok(Some(layout))
    } else if explicit {
        Err(format!(
            "{} variables were set, but no known artifacts were found",
            spec.package_name
        ))
    } else {
        Ok(None)
    }
}

fn auto_winpty_root() -> Option<PathBuf> {
    let agent = which("winpty-agent.exe")
        .or_else(|_| which("winpty-agent"))
        .ok()?;
    let bin_dir = agent.parent()?.to_path_buf();
    let parent = bin_dir.parent().unwrap_or(&bin_dir);

    if bin_dir
        .file_name()
        .map(|name| name == "bin")
        .unwrap_or(false)
        && parent.join("lib").is_dir()
    {
        Some(parent.to_path_buf())
    } else {
        Some(bin_dir)
    }
}

fn unique_dirs(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for dir in dirs {
        if !unique.iter().any(|existing| existing == dir) {
            unique.push(dir.clone());
        }
    }
    unique
}

fn find_existing(search_dirs: &[PathBuf], file_names: &[&str]) -> Option<PathBuf> {
    for dir in search_dirs {
        for file_name in file_names {
            let candidate = dir.join(file_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn pick_layout_dir(root: &Path, subdir: &str) -> PathBuf {
    let candidate = root.join(subdir);
    if candidate.is_dir() {
        candidate
    } else {
        root.to_path_buf()
    }
}

fn common_parent(first: Option<&Path>, second: Option<&Path>) -> Option<PathBuf> {
    match (first, second) {
        (Some(first), Some(second)) => {
            let first_parent = first.parent().unwrap_or(first);
            let second_parent = second.parent().unwrap_or(second);
            if first_parent == second_parent {
                Some(first_parent.to_path_buf())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    env_var(name).map(PathBuf::from)
}

fn env_bool(name: &str) -> Option<bool> {
    env_var(name).map(|value| parse_bool(name, &value))
}

fn env_var(name: &str) -> Option<OsString> {
    let prefixed = format!(
        "{}_{}",
        env::var("TARGET").unwrap().to_uppercase().replace('-', "_"),
        name
    );

    println!("cargo:rerun-if-env-changed={prefixed}");
    if let Some(value) = env::var_os(&prefixed) {
        return Some(value);
    }

    println!("cargo:rerun-if-env-changed={name}");
    env::var_os(name)
}

fn parse_bool(name: &str, value: &OsString) -> bool {
    let value = value.to_string_lossy();
    match value.as_ref() {
        "0" | "false" | "FALSE" | "no" | "NO" => false,
        "1" | "true" | "TRUE" | "yes" | "YES" => true,
        _ => panic!("{name} must be set to 0/1 or true/false"),
    }
}

fn validate_pe_arch(path: &Path, target_arch: &str, label: &str) -> Result<(), String> {
    let machine = pe_machine_type(path)?;
    if machine_matches_target(machine, target_arch) {
        Ok(())
    } else {
        Err(format!(
            "{label} at {} does not match target architecture {}",
            path.display(),
            target_arch
        ))
    }
}

fn pe_machine_type(path: &Path) -> Result<u16, String> {
    let file =
        File::open(path).map_err(|err| format!("failed to open {}: {err}", path.display()))?;
    let mut reader = BufReader::new(file);

    reader
        .seek(SeekFrom::Start(0x3c))
        .map_err(|err| format!("failed to seek in {}: {err}", path.display()))?;

    let mut offset = [0u8; 4];
    reader
        .read_exact(&mut offset)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;

    let pe_offset = u32::from_le_bytes(offset) as u64;
    reader
        .seek(SeekFrom::Start(pe_offset))
        .map_err(|err| format!("failed to seek to PE header in {}: {err}", path.display()))?;

    let mut signature = [0u8; 4];
    reader
        .read_exact(&mut signature)
        .map_err(|err| format!("failed to read PE signature from {}: {err}", path.display()))?;
    if &signature != b"PE\0\0" {
        return Err(format!("{} is not a PE file", path.display()));
    }

    let mut machine = [0u8; 2];
    reader
        .read_exact(&mut machine)
        .map_err(|err| format!("failed to read machine type from {}: {err}", path.display()))?;
    Ok(u16::from_le_bytes(machine))
}

fn machine_matches_target(machine: u16, target_arch: &str) -> bool {
    match machine {
        0x014c => target_arch == "x86",
        0x8664 => target_arch == "x86_64",
        0x01c4 => target_arch == "arm",
        0xaa64 => target_arch == "aarch64",
        0xa641 => target_arch == "x86_64",
        0xa64e => target_arch == "x86_64" || target_arch == "aarch64",
        _ => false,
    }
}
