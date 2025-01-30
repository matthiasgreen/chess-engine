# chess-engine
Chess engine for educational purposes, written in Rust. Currently around 2000 ELO.

## Features/Choices

- Bitboard representation
- Pseudo-legal move generation
- Alpha-beta pruning
- Quiescence search
- Transposition table (WIP)
- Evaluation function (WIP)

## Compiling to WebAssembly

```bash	
wasm-pack build
```

## Install with npm

```bash
npm i https://github.com/matthiasgreen/chess-engine/releases/latest/download/chess-engine-0.1.0.tgz
```
