use shogi::{Move, Position, Color, Piece, Square};
use shogi::bitboard::Factory as BBFactory;
use shogi::piece_type::PieceType;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    BBFactory::init();
    let mut pos = Position::new();

    // Position can be set from the SFEN formatted string.
    pos.set_sfen("6b2/7kl/5G2p/6pp1/8P/9/9/9/9 b 2G2rbg4s4n3l14p 1").unwrap();

    println!("digraph G {{");
    println!("// {}", if move_n(&mut pos, 3) { "tsumi" } else { "no" });
    println!("}}");
}

/**
 * When next turn is Black, returns if there is tsumi
 * When next turn is White, returns if there is not tsumi
 */
fn move_n (pos: &mut Position, ply: i8) -> bool {
    let turn = pos.side_to_move();
    let hash = calculate_hash(&pos.to_sfen());
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
            let mov = Move::Normal {from: sq, to: to, promote: false}; // TODO
            match pos.make_move(mov) {
                Ok(_) => {
                    if turn == Color::White || pos.in_check(Color::White) {
                        println!("\"{:x}\" -> \"{:x}\" [label = \"{}\"];", hash, calculate_hash(&pos.to_sfen()), mov);
                        eprintln!("{:x} {}", hash, pos);
                        let child_result = move_n(pos, ply-1);
                        if !child_result {
                            println!("// {}proofed", if turn==Color::Black {""}else{"dis"});
                            pos.unmake_move().unwrap();
                            println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor = \"{}\"];", hash, if turn==Color::Black {"box"} else {"oval"}, if turn==Color::Black{"cyan"}else{"white"});
                            return true;
                        }
                    }
                    pos.unmake_move().unwrap();
                },
                Err(_) => {}
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
                            let child_result = move_n(pos, ply-1);
                            if !child_result {
                                println!("// {}proofed", if turn==Color::Black {""}else{"dis"});
                                pos.unmake_move().unwrap();
                                println!("\"{:x}\" [shape = {}, style=\"filled\", fillcolor = \"{}\"];", hash, if turn==Color::Black {"box"} else {"oval"}, if turn==Color::Black{"cyan"}else{"white"});
                                return true;
                            }
                        }
                        pos.unmake_move().unwrap();
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
