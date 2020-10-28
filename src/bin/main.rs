use tsumeshogi_lib::dfpn::{dfpn};

use shogi::{Move, Position, Color, Piece, Square};
use shogi::bitboard::Factory as BBFactory;
use shogi::piece_type::PieceType;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

fn main() {
    // let sfen = "7nl/5+RBk1/9/6+r2/7pP/9/9/9/9 b Lb4g4s3n2l16p 1";
    // let sfen = "6b2/7kl/5G2p/6pp1/8P/9/9/9/9 b 2G2rbg4s4n3l14p 1";
    // let sfen = "8l/6s2/4S2kn/6pB1/8p/7R1/9/9/9 b Br4g2s3n3l16p 1";
    // let sfen = "8l/6s2/4+P2kn/6pB1/8p/7R1/9/9/9 b Br4g3s3n3l15p 1";
    // let sfen = "8l/6s2/7kn/4G1pB1/8p/7R1/9/9/9 b Br3g3s3n3l16p 1"; // Tsumanai yatsu
    let sfen = "9/9/7ns/8+P/9/5n2k/3G2L2/2Gbp1S2/6G2 b 3L2rbg2s2n16p 1"; // my problem
    let sfen = "8k/9/7GP/9/9/9/9/9/9 b 2r2b3g4s4n4l17p 1";
    // pos.set_sfen(std::env::args().nth(1).unwrap().as_str()).unwrap();

    solve(sfen);
}

fn solve(sfen: &str) {
    let start = Instant::now();

    BBFactory::init();

    let mut pos = Position::new();
    pos.set_sfen(sfen).unwrap();

    let result = dfpn(&mut pos);
    let elapsed_time = start.elapsed().as_secs_f64();


    println!("result: {:?}", result);
    println!(
        "nps = {} / {} = {}",
        result.nodes(),
        elapsed_time,
        f64::from(result.nodes() as u32) / elapsed_time);
    println!(
        "nps (incl. temporary) = {} / {} = {}",
        result.nodes_incl_temporary(),
        elapsed_time,
        f64::from(result.nodes_incl_temporary() as u32) / elapsed_time);
}


fn main0() {
    BBFactory::init();
    let mut pos = Position::new();

    println!("digraph G {{");
    println!("// {}", if naive_df(&mut pos, 3) { "tsumi" } else { "no" });
    eprintln!("{}", pos);
    println!("}}");
}

/**
 * Naive depth-first tsumi search
 * When next turn is Black, returns if there is tsumi
 * When next turn is White, returns if there is not tsumi
 */
fn naive_df(pos: &mut Position, ply: i8) -> bool {
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
