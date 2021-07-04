#ifndef BIBIMBAP_CHESS_GAME_H_
#define BIBIMBAP_CHESS_GAME_H_

#include <cstdint>
#include <memory>

#include "bibimbap/chess/board.h"

namespace bibimbap::chess {

enum class GameResult : std::int8_t {
  kDraw = 0,
  kWhiteVictory = 1,
  kBlackVictory = -1,
};

class Game {
public:
  // Creates the new game with the default starting position.
  Game() = default;

private:
  std::uint16_t fullmove_number_ = 0;

  Side active_player_ = Side::kWhite;

  bool white_king_side_castle_available_ = false;
  bool white_queen_side_castle_available_ = false;
  bool black_king_side_castle_available_ = false;
  bool black_queen_side_castle_available_ = false;

  // TODO(kirillbobyrev): This is a part of 50 move rule...
  // https://en.wikipedia.org/wiki/Fifty-move_rule
  std::uint16_t halfmove_clock_ = 0;
  // TODO(kirillbobyrev): 3 repetitions rule should also be a part of internal
  // state.
  // TODO(kirillbobyrev): Keep track of en-passant positions.

  // TODO(kirillbobyrev): It may be faster to use a specific implementation here
  // for better performance. Having a unique pointer means the board is not
  // located in a cache-friendly way.
  std::unique_ptr<Board> board_;
};

} // namespace bibimbap::chess

#endif  // BIBIMBAP_CHESS_GAME_H_
