use shogi::{Position, Piece, PieceType, Color, Move, Square};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Naive depth-first tsumi search
///
/// * When next turn is Black, returns if there is tsumi
/// * When next turn is White, returns if there is not tsumi
pub fn naive_df(pos: &mut Position, ply: i8) -> bool {
    let turn = pos.side_to_move();
    let hash = calculate_hash(&pos.to_sfen());
    let rb_count = pos.hand(Piece{piece_type: PieceType::Rook, color: Color::Black});
    let rw_count = pos.hand(Piece{piece_type: PieceType::Rook, color: Color::White});
    if ply<0 {
        println!("// giveup");
        println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor=\"gray\"];", hash, if turn==Color::Black {"box"} else {"oval"});
        return turn!=Color::Black;
    }
    let bb = pos.player_bb(turn);
    for sq in *bb {
        let piece = pos.piece_at(sq).unwrap();
        let tos = pos.move_candidates(sq, piece);
        for to in tos {
            for promote in [false, true].iter() {
                let mov = Move::Normal {from: sq, to, promote: *promote };
                match pos.make_move(mov) {
                    Ok(_) => {
                        if turn == Color::White || pos.in_check(Color::White) {
                            println!("\"{:x}\" -> \"{:x}\" [label = \"{}\"];", hash, calculate_hash(&pos.to_sfen()), mov);
                            eprintln!("{:x} {}", hash, pos);
                            let child_result = naive_df(pos, ply-1);
                            if !child_result {
                                println!("// {}proofed", if turn==Color::Black {""}else{"dis"});
                                pos.unmake_move().unwrap();
                                if rw_count != pos.hand(Piece{piece_type: PieceType::Rook, color: Color::White}) {
                                    eprintln!("{}", pos.to_sfen());
                                }
                                assert_eq!(hash, calculate_hash(&pos.to_sfen()), "hash is different!! {} after {}", pos, mov);
                                assert_eq!(rw_count, pos.hand(Piece{piece_type: PieceType::Rook, color: Color::White}), "rook W count is different {} after {}", pos, mov);
                                assert_eq!(rb_count, pos.hand(Piece{piece_type: PieceType::Rook, color: Color::Black}), "rook count is different {} after {}", pos, mov);
                                println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor = \"{}\"];", hash, if turn==Color::Black {"box"} else {"oval"}, if turn==Color::Black{"cyan"}else{"white"});

                                return true;
                            }
                        }
                        pos.unmake_move().unwrap();
                        assert_eq!(hash, calculate_hash(&pos.to_sfen()), "hash is different!! {} after {}", pos, mov);
                    },
                    Err(_) => {}
                }
            }
        }
    }
    for piece_type in &[PieceType::Pawn, PieceType::Lance, PieceType::Knight, PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook] {
        if pos.hand(Piece{piece_type: *piece_type, color: turn}) > 0 {
            for sq in 1..81 {
                let sq = Square::from_index(sq).unwrap();
                let mov = Move::Drop {to: sq, piece_type: *piece_type};
                match pos.make_move(mov) {
                    Ok(_) => {
                        if turn == Color::White || pos.in_check(Color::White) {
                            println!("\"{:x}\" -> \"{:x}\" [label = \"{}\"];", hash, calculate_hash(&pos.to_sfen()), mov);
                            eprintln!("{:x} {}", hash, pos);
                            let child_result = naive_df(pos, ply-1);
                            if !child_result {
                                println!("// {}proofed", if turn==Color::Black {""}else{"dis"});
                                pos.unmake_move().unwrap();
                                assert_eq!(hash, calculate_hash(&pos.to_sfen()), "hash is different!! {} after {}", pos, mov);
                                assert_eq!(rw_count, pos.hand(Piece{piece_type: PieceType::Rook, color: Color::White}), "rook W count is different {} after {}", pos, mov);
                                assert_eq!(rb_count, pos.hand(Piece{piece_type: PieceType::Rook, color: Color::Black}), "rook count is different {} after {}", pos, mov);
                                println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor = \"{}\"];", hash, if turn==Color::Black {"box"} else {"oval"}, if turn==Color::Black{"cyan"}else{"white"});
                                return true;
                            }
                        }
                        pos.unmake_move().unwrap();
                        assert_eq!(hash, calculate_hash(&pos.to_sfen()), "hash is different!! {} after {}", pos, mov);
                    },
                    Err(_) => {}
                }
            }
        }
    }
    println!("// No good move found");
    println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor=\"{}\"];", hash, if turn==Color::Black {"box"} else {"oval"}, if turn==Color::Black {"#FFFFFF"} else {"cyan"});
    return false;
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
