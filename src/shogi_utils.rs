use shogi::{Position, Move, Color, Piece, PieceType, Square, Bitboard, MoveError};
use shogi::bitboard::Factory as BBFactory;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Generates moves and hashes for every possible moves as a tsume shogi
///
/// TODO: What to do with perpetual check win?
pub fn generate_children(pos: &mut Position) -> (Vec<(u64, Move)>, u64) {
    let mut node_count_incl_temporary = 0;
    let turn = pos.side_to_move();
    let mut children: Vec<(u64, Move)> = Vec::new();

    if turn == Color::Black {
        // Optimization
        let candidates = check_candidates(pos, turn);

        // TODO: Akioute is not included
        /*
        let bb = pos.player_bb(turn);
        for sq in *bb {
            let piece = pos.piece_at(sq).unwrap();
            let tos = &pos.move_candidates(sq, piece) & candidates.get(&piece.piece_type).unwrap_or(&Bitboard::empty());
            for to in tos {
                for promote in [false, true].iter() {
                    let mov = Move::Normal { from: sq, to, promote: *promote };
                    node_count_incl_temporary += 1;
                    match pos.make_move(mov) {
                        Ok(_) => {
                            if turn == Color::White || pos.in_check(Color::White) {
                                println!(" {}", mov);
                                children.push((calculate_hash(pos), mov))
                            }
                            pos.unmake_move().unwrap();
                        },
                        Err(_) => {}
                    }
                }
            }
        }
         */
        let bb = pos.player_bb(turn);
        for sq in *bb {
            let piece = pos.piece_at(sq).unwrap();
            let tos = pos.move_candidates(sq, piece);
            for to in tos {
                for promote in [false, true].iter() {
                    let mov = Move::Normal { from: sq, to, promote: *promote };
                    node_count_incl_temporary += 1;
                    match pos.make_move(mov) {
                        Ok(_) => {
                            if turn == Color::White || pos.in_check(Color::White) {
                                children.push((calculate_hash(pos), mov))
                            }
                            pos.unmake_move().unwrap();
                        },
                        Err(MoveError::PerpetualCheckLose) => {
                            // Black's move
                            // println!("perpetual check, {}", pos);
                        }
                        Err(MoveError::PerpetualCheckWin) => {
                            // White's move
                            panic!("TODO: perpetual check win");
                        }
                        Err(MoveError::Repetition) => {panic!("In tsume moves, repetition must be always a perpetual check")}
                        Err(e) => {}
                    }
                }
            }
        }
        for (pt, bb) in candidates {
            if pos.hand(Piece { piece_type: pt, color: turn }) > 0 {
                for sq in bb {
                    let mov = Move::Drop { to: sq, piece_type: pt };
                    node_count_incl_temporary += 1;
                    match pos.make_move(mov) {
                        Ok(_) => {
                            if turn == Color::White || pos.in_check(Color::White) {
                                // println!(" {}", mov);
                                children.push((calculate_hash(pos), mov))
                            }
                            pos.unmake_move().unwrap();
                        },
                        Err(_) => {}
                    }
                }
            }
        }
    } else {
        let bb = pos.player_bb(turn);
        for sq in *bb {
            let piece = pos.piece_at(sq).unwrap();
            let tos = pos.move_candidates(sq, piece);
            for to in tos {
                for promote in [false, true].iter() {
                    let mov = Move::Normal { from: sq, to, promote: *promote };
                    node_count_incl_temporary += 1;
                    match pos.make_move(mov) {
                        Ok(_) => {
                            if turn == Color::White || pos.in_check(Color::White) {
                                children.push((calculate_hash(pos), mov))
                            }
                            pos.unmake_move().unwrap();
                        },
                        Err(_) => {}
                    }
                }
            }
        }
        for piece_type in &[PieceType::Pawn, PieceType::Lance, PieceType::Knight, PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook] {
            if pos.hand(Piece { piece_type: *piece_type, color: turn }) > 0 {
                for sq in 1..81 {
                    let sq = Square::from_index(sq).unwrap();
                    let mov = Move::Drop { to: sq, piece_type: *piece_type };
                    node_count_incl_temporary += 1;
                    match pos.make_move(mov) {
                        Ok(_) => {
                            if turn == Color::White || pos.in_check(Color::White) {
                                children.push((calculate_hash(pos), mov))
                            }
                            pos.unmake_move().unwrap();
                        },
                        Err(_) => {}
                    }
                }
            }
        }
    }
    (children, node_count_incl_temporary)
}

fn check_candidates (pos: &Position, color: Color) -> HashMap<PieceType, Bitboard> {
    let sq_king = pos.find_king(color.flip());
    if sq_king.is_none() { return HashMap::new() }
    let sq_king = sq_king.unwrap();

    // TODO Add akioute
    PieceType::iter()
        .map(|pt| {
            let ret = (pt, pos.move_candidates(sq_king, Piece{piece_type: pt, color: color.flip()}));
            ret
        })
        .collect()
}

pub fn calculate_hash(pos: &Position) -> u64 {
    calculate_hash_gen(&pos)
}

fn calculate_hash_gen<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclude_perpetual_check() {
        BBFactory::init();

        let mut pos = Position::new();
        pos.set_sfen("9/9/7+Bs/9/8k/5n1S1/2+pG2L2/2Gb3L1/6GN1 w N2rg2sn2l17p 1").unwrap();
        let move_a = Move::from_sfen("1e1f").unwrap();
        let move_b = Move::from_sfen("2f1g").unwrap();
        let move_c = Move::from_sfen("1f1e").unwrap();
        let move_d = Move::from_sfen("1g2f").unwrap();
        for _ in 0..2 {
            pos.make_move(move_a).unwrap();
            pos.make_move(move_b).unwrap();
            pos.make_move(move_c).unwrap();
            pos.make_move(move_d).unwrap();
        }
        pos.make_move(move_a).unwrap();
        pos.make_move(move_b).unwrap();
        pos.make_move(move_c).unwrap();
        let children = generate_children(&mut pos).0;
        assert_eq!(false, children.iter().any(|(_, mov)| *mov == move_d ), "Perpetual check lose is removed from children");
    }
}