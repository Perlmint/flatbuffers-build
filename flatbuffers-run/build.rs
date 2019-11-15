const fn get_flatc_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "flatc.exe";
    #[cfg(target_os = "macos")]
    return "flatc_osx";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return "flatc_linux";
}

fn main() {
    let mut path = std::env::current_dir().unwrap();
    path.push("..");
    path.push(get_flatc_name());
    println!("cargo:rustc-env=FLATC={}", path.display());
}
