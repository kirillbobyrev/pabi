use crate::chess::Position;


fn evaluate(position: &Position) -> i32 {
    let mut score = 0;

    // Piece values
    let pawn_value = 1;
    let knight_value = 3;
    let bishop_value = 3;
    let rook_value = 5;
    let queen_value = 9;

    // Count the number of each piece type
    let num_pawns = position.count_pawns();
    let num_knights = position.count_knights();
    let num_bishops = position.count_bishops();
    let num_rooks = position.count_rooks();
    let num_queens = position.count_queens();

    // Calculate the score based on the number of each piece type
    score += pawn_value * num_pawns;
    score += knight_value * num_knights;
    score += bishop_value * num_bishops;
    score += rook_value * num_rooks;
    score += queen_value * num_queens;

    score
}
