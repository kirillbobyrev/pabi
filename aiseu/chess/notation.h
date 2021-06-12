#ifndef AISEU_CHESS_NOTATION_H_
#define AISEU_CHESS_NOTATION_H_

#include <cstdint>
#include <memory>
#include <optional>
#include <unordered_map>

#include "board.h"
#include "game.h"

namespace aiseu::chess {

// Converts chess board files to numerical representation.
//
// The chessboard is divided into eight columns (files). Letters from 'a' to
// 'h' are used to identify the files. The internal representation uses numbers
// for convenience.
std::uint8_t FileToNumeric(char file);

Position AlgebraicPosition(std::string_view position);

// Construct the board from Forsyth-Edwards Notation (FEN) format.
//
// http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm#c9.7.2
Game ReadFEN();

}  // namespace aiseu::chess

#endif  // AISEU_CHESS_NOTATION_H_
