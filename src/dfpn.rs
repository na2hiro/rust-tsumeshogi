use shogi::{Position, Move, Color, PieceType, Piece, Square};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::u64::MAX;
use std::cmp::min;

pub fn dfpn(pos: &mut Position) {
    // TODO
    let mut hash_table = HashMap::new();
    let (mut p, mut d) = mid(pos, MAX - 1, MAX - 1, &mut hash_table);
    if p != MAX && d != MAX {
        println!("second time");
        let (p2, d2) = mid(pos, p, d, &mut hash_table);
        p=p2;
        d=d2;
    }

    println!("result: {} (p={}, d={})", if d ==MAX {"tsumi"} else if p==MAX {"futsumi"} else {"?"}, p, d);
}

fn mid(pos: &mut Position, phi: u64, delta: u64, hash_table: &mut HashMap<u64, (u64, u64)>) -> (u64, u64) {
    // println!("{}", pos);
    let turn = pos.side_to_move();
    // look up hash
    let hash = calculate_hash(pos);
    let (p, d) = look_up_hash(&hash, &hash_table);
    if phi < *p && delta < *d {
        return (*p, *d);
    }
    let mut children: Vec<(u64, Move)> = Vec::new();

    // Generate moves
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
        if pos.hand(Piece{piece_type: *piece_type, color: turn}) > 0 {
            for sq in 1..81 {
                let sq = Square::from_index(sq).unwrap();
                let mov = Move::Drop {to: sq, piece_type: *piece_type};
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

    // Leaf node
    if children.is_empty() {
        // println!("LEAF TSUMI");
        put_in_hash(hash, MAX, 0, hash_table);
        return (MAX, 0);
    }

    // println!("{:?}", children);

    // 3. Prevent cycle
    put_in_hash(hash, phi, delta, hash_table);

    // 4 Iterative deepening
    loop {
        let md = min_delta(&children, hash_table);
        let mp = sum_phi(&children, hash_table, &pos);
        if phi <= md || delta <= mp {
            put_in_hash(hash, md, mp, hash_table);
            return (md, mp);
        }
        let (best_move, phi_c, delta_c, delta_2) = select_child(&children, hash_table);
        let phi_n_c = if phi_c == MAX-1 {
            MAX
        } else if delta >= MAX-1 {
            MAX-1
        } else {
            delta + phi_c - mp
        };
        let delta_n_c = if delta_c == MAX - 1 {
            MAX
        } else {
            if delta_2 == MAX { phi } else { min(phi, delta_2+1) }
        };
        pos.make_move(*best_move).unwrap();
        mid(pos, phi_n_c, delta_n_c, hash_table);
        pos.unmake_move().unwrap();
    }
}

fn select_child<'a>(children: &'a Vec<(u64, Move)>, hash_table: &mut HashMap<u64, (u64, u64)>) -> (&'a Move, u64, u64, u64) {
    let mut delta_c = MAX;
    let mut delta_2 = MAX;
    let mut phi_c = MAX; // ok?
    let mut best_move = Option::None;

    for (hash, mov) in children {
        let (p, d) = look_up_hash(&hash, hash_table);
        if *d < delta_c {
            best_move = Option::Some(mov);
            delta_2 = delta_c;
            phi_c = *p;
            delta_c = *d;
        } else if *d < delta_2 {
            delta_2 = *d;
        }
        if *p == MAX {
            return (best_move.unwrap(), phi_c, delta_c, delta_2);
        }
    }
    return (best_move.unwrap(), phi_c, delta_c, delta_2);
}

fn look_up_hash<'a>(hash: &u64, hash_table: &'a HashMap<u64, (u64, u64)>) -> &'a (u64, u64) {
    hash_table.get(hash).unwrap_or(&(1u64, 1u64))
}

fn put_in_hash(hash: u64, phi: u64, delta: u64, hash_table: &mut HashMap<u64, (u64, u64)>) {
    hash_table.insert(hash, (phi, delta));
}

fn min_delta(children: &Vec<(u64, Move)>, hash_table: &mut HashMap<u64, (u64, u64)>) -> u64 {
    let mut min_delta = MAX;
    for (hash, _) in children {
        let (_, d) = look_up_hash(&hash, &hash_table);
        min_delta = min(min_delta, *d);
    }
    min_delta
}

fn sum_phi(children: &Vec<(u64, Move)>, hash_table: &mut HashMap<u64, (u64, u64)>, pos: &Position) -> u64 {
    let mut sum = 0;
    for (hash, mov) in children {
        let (p, _) = look_up_hash(&hash, &hash_table);
        if *p == MAX {
            // println!("Hmm going to overflow? {} {}", mov, &pos);
            return MAX;
        }
        sum += p;
    }
    sum
}

fn calculate_hash(pos: &Position) -> u64 {
    let str = format!("{}", pos);
    calculate_hash_gen(&str)
}

fn calculate_hash_gen<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
