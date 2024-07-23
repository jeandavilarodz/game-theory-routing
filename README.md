## Description ###
This project implements a simulation for opportunistic routing in space scenarios
based on game theory concepts using basic assumptions of NASA's DTN protocol

## Install ##

This project is written in webassembly and Rust, we need to install the Rust toolchain for
WebAssembly using rustup:

```
rustup target add wasm32-unknown-unknown
```

Then we need to install the WASM bindings:

```
cargo install --locked wasm-bindgen-cli
```

We build the project with a tool called Trunk, to prevent us from writing Javascript:

```
cargo install --locked trunk
```

To build the desktop application, we will need to install Tauri CLI:

```
cargo install tauri-cli
```


## Building ##

To build the web application we can go to the simulation folder and use Trunk to build:

```
cd simulation
trunk build --release
```

This will create the WASM binary in the target directory.


## Building the Desktop Application ##

We can use Tauri cli to build the desktop application:

```
cargo tauri build
```

This will also call the Trunk binary to compile the simulation and package it up into an app.


## Attributes ###

Orbit icons created by Freepik - Flaticon
