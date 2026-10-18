fn main() {
    slint_build::compile("ui/timer.slint")
        .expect("Slint-Oberfläche konnte nicht kompiliert werden");
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/clock.ico")
        .set("ProductName", "Timer")
        .set("FileDescription", "Timer")
        .compile()
        .expect("Windows-Ressourcen konnten nicht kompiliert werden");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let executable_dir = output.ancestors().nth(3).unwrap();
    std::fs::copy("assets/ding.wav", executable_dir.join("ding.wav"))
        .expect("ding.wav konnte nicht neben die EXE kopiert werden");
    println!("cargo:rerun-if-changed=assets/clock.ico");
    println!("cargo:rerun-if-changed=assets/clock.svg");
    println!("cargo:rerun-if-changed=assets/ding.wav");
    println!("cargo:rerun-if-changed=assets/font.ttf");
}
