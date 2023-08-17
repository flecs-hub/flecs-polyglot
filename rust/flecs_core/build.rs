use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=flecs.h");
    println!("cargo:rerun-if-changed=flecs.c");

    // Get rid of the warning about unused command line arguments from emcc
    std::env::set_var("CFLAGS", "-Wno-unused-command-line-argument");

    let mut bindings = bindgen::Builder::default()
		.header("flecs.h")
		.clang_arg("-fvisibility=default")	// Necessary for Emscripten target.
		.generate_comments(false)
		.layout_tests(false)
		// Tell cargo to invalidate the built crate whenever any of the
		// included header files changed.
		.parse_callbacks(Box::new(bindgen::CargoCallbacks));

    // Export as JS file as ES6 Module by adding emscripten flag
    println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
    println!("cargo:rustc-link-arg=-sMODULARIZE=1");

    // Standard library include path
    // To support all platforms we should use the emsdk sysroot itself for the include path.
    let emsdk = env::var("EMSDK").unwrap();
    let emsdk_include_path = format!("{}/upstream/emscripten/cache/sysroot/include", emsdk);
    let include_path = env::var("STDLIB").unwrap_or(emsdk_include_path.to_string()).to_string();
    let include_flag = String::from("-I") + &include_path[..include_path.len()];
    println!("Used Include Path: {}", include_path);

    bindings = bindings.clang_arg(include_flag);

    let bindings = bindings
		.generate()
		.expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("src/bindings.rs"))
		.expect("Couldn't write bindings!");

    // Compile Flecs
    cc::Build::new()
        .include("flecs.h")
        .file("flecs.c")
        .compile("flecs_core");
}
