# Architecture

This document describes high-level architecture of Pabi. If you want to get
familiar with the code and understand the project structure, this is a great
starting point.

## Overview

## Code map

### [`src/`](/src/)

The source directory contains code for the engine driver implemented in Rust.

#### [`src/chess/`](/src/chess/)

#### [`src/search/`](/src/search/)

#### [`src/evaluation/`](/src/evaluation/)

#### [`src/interface/`](/src/interface/)

### [`training/`](/training/)

### [`tests/`](/tests/)

### [`benches/`](/benches/)

### [`fuzz/`](/fuzz/)

### [`generated/`](/generated/)

Pre-computed constants (such as [Magic Bitboards], [Vector Attacks] and [Zobrist
hashing] table) speed up move generation and search. This data can be calculated
at build time or startup instead but the drawbacks are:

- Build time (compile-time Rust code can not make use of most built
  infrastructure) or runtime (warm-up time) overhead
- Maintenance cost
- Losing opportunities for the compiler to do better optimizations

Hence, these values are calculated once and checked into the source tree as Rust
arrays. These constants shouldn't change over time.

[Magic Bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[vector attacks]: https://www.chessprogramming.org/Vector_Attacks
[Zobrist hashing]: https://www.chessprogramming.org/Zobrist_Hashing
