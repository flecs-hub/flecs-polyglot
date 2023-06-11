fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=build.rs");

    // Get rid of the warning about unused command line arguments from emcc
    std::env::set_var("CFLAGS", "-Wno-unused-command-line-argument");

    // Compile Flecs
    cc::Build::new()
        .include("flecs.h")
        .file("flecs.c")
        .compile("flecs_core");
}
