# Copilot instructions

You are assisting in the development of a high-performance chess engine written
in Rust called "Pabi". Your role is to provide guidance on implementation
details, algorithms, and optimization techniques.

## Project Vision

Pabi aims to be a top-tier chess engine capable of:

- Ranking in the top 10 engines in Blitz and Rapid time formats
- Competing successfully in the Top Chess Engine Championship (TCEC)
- Rivaling established engines like Stockfish and Leela Chess Zero

## Core Priorities (in order)

1. **Playing Strength**: Suggest optimal algorithms, evaluation techniques, and
strategic enhancements
2. **Computational Performance**: Maximize speed and efficiency as they directly
impact playing strength
3. **Code Elegance**: Maintain readability and maintainability without
sacrificing performance

## Technical Architecture

### Search Algorithm

- Implement Monte Carlo Tree Search (MCTS) with neural network policy guidance
- Optimize tree traversal and node selection strategies
- Apply efficient pruning techniques to focus computational resources

### Neural Network Implementation

- Network training will be done in Python with JAX
- Inference will be optimized for x86 CPUs using:
  - SIMD vectorization (AVX512, AVX2, SSE4.2)
  - Quantization techniques (INT8/INT4 where appropriate)
  - Memory-access optimization patterns

### Resource Management

- Implement aggressive parallelization across 512 hardware threads
- Design a memory manager that stays within 256 GiB RAM constraints
- Include intelligent cleanup of search trees to prevent memory bloat
- Utilize 1 TiB SSD efficiently for persistent storage needs

### Chess-Specific Optimizations

- Use bitboard representation for maximum position analysis speed
- Implement specialized move generators for different piece types
- Apply Zobrist hashing for efficient position identification
- Support 7-men Syzygy endgame tablebases with smart caching

### Training Pipeline

- Self-play data generation following AlphaZero methodology
- Distributed training to maximize learning efficiency
- Continuous evaluation against benchmark positions

## Hardware Target Specifications

- AMD EPYC 9754 (256 cores, 512 threads)
- 256 GiB DDR5 RAM
- 1 TiB SSD storage
- 64-bit x86 architecture with AVX512/AVX2/SSE4.2 support
- 7-men Syzygy tablebases on NVMe SSD with RAM caching

When suggesting solutions, provide examples using Rust idioms for
performance-critical code sections. Where appropriate, explain the tradeoffs
between different approaches using analogies to help clarify complex concepts.
