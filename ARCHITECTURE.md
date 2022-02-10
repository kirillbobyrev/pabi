# Architecture

This document describes high-level architecture of Pabi. If you want to get
familiar with the code and understand the project structure, this is a great
starting point.

## Overview

## Code map

### `src/`

#### `src/chess/`

#### `src/search/`

#### `src/evaluation/`

#### `src/interface/`

### `training/`

### `tests/`

### `benches/`

### `fuzz/`

### `generated/`

Move generator and search benefit from pre-computed constants such as [Magic
Bitboards], [Vector Attacks] and [Zobrist hashing] table. This data can be
calculated at build time or startup instead but the drawbacks are:

- Build time (compile-time Rust code can not make use of most built
  infrastructure) or runtime (warm-up time) overhead
- Maintenance cost
- Losing opportunities for the compiler to do better optimizations

Hence, these values are calculated once and checked into the source tree as Rust
arrays. These constants shouldn't change over time.

[Magic Bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[vector attacks]: https://www.chessprogramming.org/Vector_Attacks
[Zobrist hashing]: https://www.chessprogramming.org/Zobrist_Hashing
