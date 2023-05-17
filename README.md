# Holon
A P2P node that wraps Urbit and gives it scalable properties.

## Getting started

### 1. Setup rust

```zsh
# install rustup
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
# make sure to have c-compiler
xcode-select --install
# Verify installation
rustc --version

# to update rustup for future releases
rustup update
```

### 2. Build the project with cargo

Build for dev:
```zsh
cargo build
cargo run
```

Build for production:
```zsh
cargo build --release
```

### 3. Running the node

The below command will print the debug cli
```zsh
cargo run 
```
