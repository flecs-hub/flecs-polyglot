fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=build.rs");
    
    cc::Build::new()
        .include("flecs.h")
        .file("flecs.c")
        .compile("flecs_core");
}
