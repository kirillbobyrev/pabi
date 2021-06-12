#ifndef AISEU_CHESS_BITBOARD_H_
#define AISEU_CHESS_BITBOARD_H_

#include "aiseu/chess/board.h"

namespace aiseu::chess {

class Bitboard {
 public:
 private:
   std::uint64_t bits_;
};

}  // namespace aiseu::chess

#endif  // AISEU_CHESS_BITBOARD_H_
