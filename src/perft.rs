//! [Perft] (*per*formance *t*esting) is a technique for checking correctness of
//! move generation (tested functions are generate move, make move and unmake
//! move).
//!
//! [Perft]: https://www.chessprogramming.org/Perft
// TODO: Also implement divide and use <https://github.com/jniemann66/juddperft> to validate the
// results.
// TODO: Maybe use python-chess testset of perft moves:
// https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
// TODO: Compare with other engines and perft generattors, e.g. Berserk,
// shakmaty.
