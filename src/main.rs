use shogi::{Move, Position, Color};
use shogi::bitboard::Factory as BBFactory;

fn main() {
    BBFactory::init();
    let mut pos = Position::new();

    // Position can be set from the SFEN formatted string.
    pos.set_sfen("7k1/9/5G1G1/9/9/9/9/9/9 b 2r2b2g4s4n4l18p 1").unwrap();

    move_n(&mut pos, 3);
}

fn move_n (pos: &mut Position, ply: u8) {
    let turn = pos.side_to_move();
    let bb = pos.player_bb(turn);
    for sq in *bb {
        let piece = pos.piece_at(sq).unwrap();
        let tos = pos.move_candidates(sq, piece);
        for to in tos {
            let mov = Move::Normal {from: sq, to: to, promote: false}; // TODO
            match pos.make_move(mov) {
                Ok(_) => {
                    if turn == Color::White || pos.in_check(Color::White) {
                        println!("{}", pos);
                        if ply>1 {
                            move_n(pos, ply-1);
                        }
                    }
                    pos.unmake_move().unwrap();
                },
                Err(_) => {}
            }
        }
    }
}
