# Typegraph

Typegraph is a Rust library for creating type-level graphs of Rust types. It enables you to visualize and analyze the relationships between your types at compile time.

## Overview

Typegraph allows you to build graphs that represent your Rust types and their relationships. These graphs can be used for documentation, analysis, or simply as a way to better understand your codebase structure.

### Key Features

- **Type-level graphs**: Create graphs that represent your Rust types at compile time
- **Automatic graph generation**: Use derive macros to automatically generate type graphs
- **Graph visualization**: Export graphs to Graphviz format for visualization
- **Flexible graph structure**: Support for various relationship types between types
- **No_std support**: Works in both `std` and `no_std` environments

## Getting Started

Add Typegraph to your Cargo.toml:

```toml
[dependencies]
typegraph = "0.1.0"
```

### Basic Usage

```rust
use typegraph::{typegraph, Typegraph};

#[typegraph]
pub struct MyType {
    field: u32,
    other: String,
}

#[typegraph]
impl MyType {
    pub fn new() -> Self {
        Self {
            field: 42,
            other: "hello".to_string(),
        }
    }
}
```

### Graph Visualization

```rust
use typegraph::{typegraph, Graphviz};

#[typegraph]
pub struct Foo {
    bar: Bar,
}

#[typegraph]
pub struct Bar {
    value: u32,
}

// Generate Graphviz output
let graph = typegraph::Resolve<Foo>;
let output = graph.render();
println!("{}", output);
```

## Features

- **value**: Enable value-level representations of types
- **graphviz**: Enable Graphviz export functionality
- **std**: Enable std-specific types and functionality
- **inert**: Enable inert type-level computation

## Documentation

For detailed documentation, see the [docs.rs page](https://docs.rs/typegraph).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License, Version 2.0.

