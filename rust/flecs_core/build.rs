use std::path::{PathBuf, Path};

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=flecs.h");
    println!("cargo:rerun-if-changed=flecs.c");

    let target = std::env::var("TARGET").unwrap();
    if target.contains("emscripten") {
        // Get rid of the warning about unused command line arguments from emcc
        std::env::set_var("CFLAGS", "-Wno-unused-command-line-argument");
    };

    // // Bindgen
    // let bindings = bindgen::Builder::default()
    //     .header(Path::new("flecs.h").to_str().unwrap())
    //     .generate()
    //     .expect("Unable to generate bindings");

    // let out_path = PathBuf::from("./src");
    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");

    // Compile Flecs
    cc::Build::new()
        .include("flecs.h")
        .file("flecs.c")
        .compile("flecs_core");
}
