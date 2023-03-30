fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lua/*");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let compiler = mlua::Compiler::new()
        .set_coverage_level(0)
        .set_debug_level(0)
        .set_optimization_level(2);
    for path in std::fs::read_dir("src/lua").unwrap() {
        let de = path.unwrap();
        drop(std::fs::write(
            format!("{}/{}",out_dir,de.file_name().to_string_lossy()),
        compiler.compile(
                std::fs::read_to_string(de.path().display().to_string()).unwrap()
                )
        ));
    }
}