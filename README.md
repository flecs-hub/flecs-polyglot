# Flecs Polyglot
*⚠️ Warning ⚠️ - This repository is under construction.*

## Introduction
A universal scripting API for [flecs](https://github.com/SanderMertens/flecs) on all languages that compile to WebAssembly. This is achieved through WebAssembly module linking (on the web through Emscripten, on native through Wasmtime), or native dynamic linking for a zero runtime option.

[flecs](https://github.com/SanderMertens/flecs) is a blazing-fast, cache-friendly, portable, entity component system written in C that supports relationships, hierarchies, and more! 

## Goals
Provide one universal set of scripting bindings using powerful technologies such as WebAssembly with minimal overhead during interop with the host program. Fast memory mapped access to C struct data is used instead of reflection. 

# Usage (WIP)
Implementation / Project Scaffolding CLI can be found in the Toxoid Engine repository: https://github.com/toxoidengine/toxoid

# Supported Platforms
- Web
- Mobile
- Desktop (Windows / Linux / OS X)

## Supported Languages
- Rust
- C / C++
- TypeScript / JavaScript
- C# (Not yet)
- AssemblyScript (Not yet)
- Lua / LuaJIT (Not yet)
- Kotlin (Not yet)
- Java (Not yet)
- Zig (Not yet)
- Swift (Not yet)
- Haxe (Not yet)
- Rhai (Not yet)
- Go / TinyGo (Not yet)