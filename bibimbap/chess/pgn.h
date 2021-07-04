#ifndef BIBIMBAP_CHESS_PGN_H_
#define BIBIMBAP_CHESS_PGN_H_

#include "bibimbap/chess/board.h"

namespace bibimbap::chess {

// Construct the game from Portable Game Notation (PGN) format.
//
// http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm
void ReadPGN(std::string_view pgn);

}  // namespace bibimbap

#endif  // BIBIMBAP_CHESS_PGN_H_
