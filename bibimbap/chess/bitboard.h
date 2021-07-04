#ifndef BIBIMBAP_CHESS_BITBOARD_H_
#define BIBIMBAP_CHESS_BITBOARD_H_

#include "bibimbap/chess/board.h"

namespace bibimbap::chess {

class Bitboard {
 public:
 private:
   std::uint64_t bits_;
};

}  // namespace bibimbap::chess

#endif  // BIBIMBAP_CHESS_BITBOARD_H_
