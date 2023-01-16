build: 
	cd ./ts && npx spack
	cd ./rust/flecs_core && cargo build --target=wasm32-unknown-emscripten