<h1 align="center">
  Pabi
</h1>

[![Codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Dependencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml)

Pabi is a modern chess engine that is currently under development.

> [!WARNING]
> This engine is still very early in the development phase and is not in a
> functional state yet.

## Goals

Pabi is inspired by existing Chess and Go engines (mainly [lc0] and [KataGo]),
and the research that they are based on ([AlphaZero], [MuZero] and [MCTS]). Pabi
strives to be a high-quality modern engine.

Pabi strives to be **strong**. The ultimate goal is to enter the lists of top
chess engines and participate in tournaments, such as and [TCEC] and [CCC].

<!-- TODO: User interface: supported commands + UCI options -->
<!-- Describe high-level features -->

## Recipes

Most commands for development, building the engine, testing, checking for errors
and fuzzing it are supported as [just](https://github.com/casey/just) recipes.
See [justfile](/justfile) for a complete list of frequently used commands.

## Code map

For easier code navigation, see
<https://kirillbobyrev.github.io/pabi/docs/pabi/>

## [Milestones]

- [ ] [Proof of Concept]
- [ ] [Stable]
- [ ] [Strong]

[AlphaZero]: https://en.wikipedia.org/wiki/AlphaZero
[MuZero]: https://deepmind.google/discover/blog/muzero-mastering-go-chess-shogi-and-atari-without-rules/
[CCC]: https://www.chess.com/computer-chess-championship
[KataGo]: https://github.com/lightvector/KataGo
[MCTS]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[Proof of Concept]: https://github.com/kirillbobyrev/pabi/milestone/1
[Stable]: https://github.com/kirillbobyrev/pabi/milestone/2
[Strong]: https://github.com/kirillbobyrev/pabi/milestone/3
[TCEC]: https://tcec-chess.com/
[lc0]: https://lczero.org/
