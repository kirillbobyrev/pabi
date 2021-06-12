#ifndef AISEU_CHESS_PGN_H_
#define AISEU_CHESS_PGN_H_

#include "aiseu/chess/board.h"

namespace aiseu::chess {

// Construct the game from Portable Game Notation (PGN) format.
//
// http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm
void ReadPGN(std::string_view pgn);

}  // namespace aiseu

#endif  // AISEU_CHESS_PGN_H_
