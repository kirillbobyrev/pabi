# pabi-brain

Code for training Neural Network-based static evaluation for chess positions.

The idea is to use the approach most successful chess engines utilize: distill
Leela Chess Zero's network using the data from its training. The resulting
network should ideally be very close to the original version, have very high
inference speed and could be ported to CPU easily.

Pabi doesn't strive to repeat [Stockfish NNUE] architecture and design, but it
has a lot of very valuable lessons that should definitely be taken into
consideration.

[Stockfish NNUE]: https://github.com/official-stockfish/nnue-pytorch/blob/master/docs/nnue.md
