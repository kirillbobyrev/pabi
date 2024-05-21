# Contributing guide

This document describes high-level architecture of Pabi. If you want to get
familiar with the code and understand the project structure, this is a great
starting point.

## Scope

Implementing a *general-purpose* chess engine that would truly excel in
different domains, such as playing with humans, being able to adjust the
strength, providing the best possible analysis for a position when given a lot
of time and, finally, playing well against other engines is an enormous task,
especially for one person. Therefore, it's crucial to choose the right scope and
design the engine with chosen limitations in mind.

Pabi chooses to be the best possible version of an engine that does well against
other chess engines in online tournaments. That means that it will prioritize
performance under the constraints put by the rules and environments of such
tournaments. Most important tournament organizers are
[TCEC](https://tcec-chess.com/) and
[CCCC](https://www.chess.com/computer-chess-championship), and the most
prominent rating to date is [CCRL](https://computerchess.org.uk/ccrl/). The
first goal is reaching 3000 ELO on the rating lists that are accepted by the
organizers of these tournaments.

Most of the competitions are in relatively fast time controls (Blitz, Bullet,
Rapid) with some exceptions in relatively short classical formats. These are the
[CCRL rules](https://computerchess.org.uk/ccrl/404/about.html) and [TCEC
rules](https://wiki.chessdom.org/Rules). In both rule sets the engines are given
opening books selected by the organizers and unknown to the participants, and
the engines start the game in pre-determined positions which creates space for
unbalance leading to decisive results.

In all cases, the testing environment's CPU is of x86_64 architecture, which is
very important because of PEXT and PDEP instructions that significantly increase
the performance of move generators. Also, the processors have multiple cores,
sometimes there as many as 256 cores as in CCC22:

```
CPUs | 2 x AMD EPYC 7H12
GPU | 2x A100 (40 GB GPU memory)
Cores | 256 cores (128 physical)
RAM | 512GB DIMM DDR4 2933 MHz (0.3 ns)
SSD | 2x Micron 5210 MTFD (2TB) in RAID1
OS | CentOS 8
```

This requires the engine to be very good at utilizing multi-threading.

Other design choices are deliberate focus on performance over most things
(except simplicity and clarity), such as error recovery (there should be minimal
one: if the error happened and the engine can reject the input, it will reject
the input and continue working) and support for arcane environments.

## Recipes

Most commands for development, building the engine, testing, checking for errors
and fuzzing it are supported as [just](https://github.com/casey/just) recipes.
See [justfile](/justfile) for a complete list of frequently used commands.

## Code map

### [`src/`](/src/)

The source directory contains code for the engine driver implemented in Rust.

#### [`src/chess/`](/src/chess/)

Contains implementation of the chess environment: [Bitboard]-based board
representation, move generation and position parsing. This is the core of the
engine: a fast move generator and convenient board implementation are crucial
for engine's performance.

#### [`src/evaluation/`](/src/evaluation/)

Contains code that extracts features from a given position and runs "static"
[position evaluation]: a Neural Network considers just a single position and
computes the score that determines how good it is for the player that is going
to make the next move.

The development of the Neural Network model is in another repo:
[kirillbobyrev/pabi-brain](https://github.com/kirillbobyrev/pabi-brain). The
Rust code only runs inference of an already trained model.

#### [`src/search/`](/src/search/)

[Monte-Carlo Tree Search]-based best move [search]. This is "dynamic" position
evaluation that considers possible continuations from a given root position,
evaluates these continuations and computes the final position evaluation, based
on the most prominent lines that can be played out of it. MCTS generates a large
number of playouts following the tree policy and adjusts the score returned by
static evaluation.

#### [`src/interface/`](/src/interface/)

Provides a way for the engine to [communicate with a front-end] (for human play)
or a tournament manager (for playing against other engines). Implements
[Universal Chess Interface] (UCI) protocol. UCI is not targeting human users but
it is not hard to learn and can be an easy way to debug some parts of the engine
manually, so it is also extended with several useful commands (for example, `go
perft`).

### [`tests/`](/tests/)

Tests the engine through public interfaces. Most tests should go here, unit
tests are only valuable for testing private functions that aren't exposed but
are still not trivial.

### [`benches/`](/benches/)

Performance is crucial for a chess engine. This directory contains a number of
performance regression tests that should be frequently run to ensure that the
engine is not becoming slower. Patches affecting performance should have
benchmark result deltas in the description.

### [`fuzz/`](/fuzz/)

[Fuzzers] complement the existing tests by generating random inputs and trying
to increase the coverage. Plenty of bugs can be caught by even relatively simply
fuzzers: writing and running them is highly encouraged.

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

[Bitboard]: https://www.chessprogramming.org/Bitboards
[Monte-Carlo Tree Search]: https://www.chessprogramming.org/Monte-Carlo_Tree_Search
[search]: https://www.chessprogramming.org/Search
[position evaluation]: https://www.chessprogramming.org/Evaluation
[Fuzzers]: https://en.wikipedia.org/wiki/Fuzzing
[communicate with a front-end]: https://www.chessprogramming.org/User_Interface
[Universal Chess Interface]: http://wbec-ridderkerk.nl/html/UCIProtocol.html
[Magic Bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[vector attacks]: https://www.chessprogramming.org/Vector_Attacks
[Zobrist hashing]: https://www.chessprogramming.org/Zobrist_Hashing
