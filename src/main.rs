use shogi::{Move, Position, Color};
use shogi::bitboard::Factory as BBFactory;

fn main() {
    BBFactory::init();
    let mut pos = Position::new();

    // Position can be set from the SFEN formatted string.
    pos.set_sfen("7k1/9/5G1G1/9/9/9/9/9/9 b 2r2b2g4s4n4l18p 1").unwrap();

    move_black(&mut pos);
}

fn move_black (pos: &mut Position) {
    let bb = pos.player_bb(Color::Black);
    for sq in *bb {
        let piece = pos.piece_at(sq).unwrap();
        let tos = pos.move_candidates(sq, piece);
        for to in tos {
            let mov = Move::Normal {from: sq, to: to, promote: false};
            pos.make_move(mov).unwrap();
            if pos.in_check(Color::White) {
                println!("{}", pos);
                move_white(pos);
            }
            pos.unmake_move().unwrap();
        }
    }
}

fn move_white(pos: &mut Position) {
    let bb = pos.player_bb(Color::White);
    for sq in *bb {
        let piece = pos.piece_at(sq).unwrap();
        let tos = pos.move_candidates(sq, piece);
        for to in tos {
            let mov = Move::Normal {from: sq, to: to, promote: false};
            match pos.make_move(mov) {
                Ok(_) => {
                    println!("{}", pos);
                    pos.unmake_move().unwrap();
                },
                Err(_) => {}
            }
        }
    }
}