# Chess Engine

## Introduction

### Motivations

- Love for chess
- Fascinating algorithms
- Want to learn rust, good project to start

### Scope

- Move generation, etc...
- Playable interface with FEN/PGN I/O
- Evaluation/Search
- Heavy optimisation/threading
- Reinforcement Learning

### Goal

- Elo of 2500

## Part 1: Move generation

### Approaching the problem

Will need to be extremely optimized due to millions of calls.
Not starting with a naïve approach or it will need to be completely rewritten

- Postition representation
  - Board representation
    - Bitboards
  - Flags for castling and active color
  - En passant square
  - Half move counter for 50 turn draw
  - Repetition? Not quite yet
  - What else to store? Visit counter? Irreversible last move? Check/mate/draw status?
  - Space efficient:
    - 16 bitboards for all pieces
    - 1 bitboard for en passant
    - 8 bits of flags
    - 8 bits for half-move counter 
  - Time efficient
    - bitwise operations on bitboards for set manipulation
    - other efficient operations like popcount
- Move representation
  - Decision to not generate and apply moves, but instead new positions directly
    - Moves are not that useful for the engine
    - Moves can quickly be computed from position delta