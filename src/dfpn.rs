extern crate wasm_bindgen;

use shogi::{Position, Move, Color, PieceType, Piece, Square, Bitboard};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::u64::MAX;
use std::cmp::{min, max};
use wasm_bindgen::prelude::*;

use serde::{Serialize, Deserialize};

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    is_tsumi: bool,
    moves: Vec<String>,
    ply_to_leaf: PlyToLeaf,
    nodes: u64,
    nodes_incl_temporary: u64,
}
impl SearchResult {
    pub fn nodes(&self) -> u64 {
        self.nodes
    }
    pub fn nodes_incl_temporary(&self) -> u64 {
        self.nodes_incl_temporary
    }
}

type PlyToLeaf = i16;
type HashTable = HashMap<u64, (u64, u64, PlyToLeaf)>;

pub fn dfpn(pos: &mut Position) -> SearchResult {
    let mut hash_table: HashTable = HashMap::new();
    let (mut p, mut d, mut ply_to_leaf, mut cnt, mut cnt_tmp) = mid(pos, MAX - 1, MAX - 1, &mut hash_table);
    if p != MAX && d != MAX {
        // println!("second time");
        let (p2, d2, ply_to_leaf2, cnt2, cnt_tmp2) = mid(pos, p, d, &mut hash_table);
        p=p2;
        d=d2;
        ply_to_leaf = ply_to_leaf2;
        cnt+=cnt2;
        cnt_tmp+= cnt_tmp2;
    }

    SearchResult {
        is_tsumi: d==MAX,
        moves: get_moves(pos, p==0, hash_table),
        ply_to_leaf,
        nodes: cnt,
        nodes_incl_temporary: cnt_tmp,
    }
}

fn mid(pos: &mut Position, phi: u64, delta: u64, hash_table: &mut HashTable) -> (u64, u64, PlyToLeaf, u64, u64) {
    // look up hash
    let hash = calculate_hash(pos);
    let (p, d, ply_to_end) = look_up_hash(&hash, &hash_table);
    if phi < *p && delta < *d {
        return (*p, *d, *ply_to_end, 1, 1);
    }

    // println!("{}", pos);

    let (children, mut node_count_incl_temporary) = generate_children(pos);

    // Leaf node
    if children.is_empty() {
        // println!("LEAF TSUMI");
        put_in_hash(hash, MAX, 0, 0, hash_table);
        return (MAX, 0, 0, 1, node_count_incl_temporary);
    }

    // println!("{:?}", children);

    // 3. Prevent cycle
    put_in_hash(hash, phi, delta, 0, hash_table);

    // 4 Iterative deepening
    let mut node_count = 0;
    loop {
        let (md, ply_to_leaf) = min_delta(&children, hash_table);
        let mp = sum_phi(&children, hash_table);
        if phi <= md || delta <= mp {
            // println!("{}, {}, {}, {}", pos, md, mp, ply_to_leaf);
            put_in_hash(hash, md, mp, ply_to_leaf, hash_table);
            return (md, mp, ply_to_leaf, node_count, node_count_incl_temporary);
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
        let (_, _, _, cnt, cnt_incl_temporary) = mid(pos, phi_n_c, delta_n_c, hash_table);
        node_count += cnt;
        node_count_incl_temporary += cnt_incl_temporary;
        match pos.unmake_move() {
            Ok(_) => {},
            Err(_) => panic!("{}, {:?}", pos, pos.move_history().last().unwrap()),
        }
    }
}

fn generate_children(pos: &mut Position) -> (Vec<(u64, Move)>, u64) {
    let mut node_count_incl_temporary = 0;
    let turn = pos.side_to_move();
    let mut children: Vec<(u64, Move)> = Vec::new();

    // Generate moves
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
                        Err(_) => {}
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

fn select_child<'a>(children: &'a Vec<(u64, Move)>, hash_table: &mut HashTable) -> (&'a Move, u64, u64, u64) {
    let mut delta_c = MAX;
    let mut delta_2 = MAX;
    let mut phi_c = MAX; // ok?
    let mut best_move = Option::None;

    for (hash, mov) in children {
        let (p, d, _) = look_up_hash(&hash, hash_table);
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

fn look_up_hash<'a>(hash: &u64, hash_table: &'a HashTable) -> &'a (u64, u64, PlyToLeaf) {
    hash_table.get(hash).unwrap_or(&(1u64, 1u64, 0))
}

fn put_in_hash(hash: u64, phi: u64, delta: u64, ply_to_leaf: PlyToLeaf, hash_table: &mut HashTable) {
    hash_table.insert(hash, (phi, delta, ply_to_leaf));
}

/// Current is OR -> child is AND: min(pn)
/// 0 -> one of the children leads to tsumi
/// MAX -> all of the children are not tsumi
/// Current is AND -> child is OR: min(dn)
/// 0 -> one of the children leads to not tsumi
/// MAX -> all of the children are tsumi
fn min_delta(children: &Vec<(u64, Move)>, hash_table: &mut HashTable) -> (u64, PlyToLeaf) {
    let mut min_delta = MAX;
    let mut ply_to_leaf_min = 32767;
    let mut ply_to_leaf_max = -1;
    for (hash, _) in children {
        let (_, d, ply_to_leaf_c) = look_up_hash(&hash, &hash_table);
        min_delta = min(min_delta, *d);
        if *d==0 {
            ply_to_leaf_min = min(ply_to_leaf_min, *ply_to_leaf_c + 1);
        }
        if *d == MAX {
            ply_to_leaf_max = max(ply_to_leaf_max, *ply_to_leaf_c + 1);
        }
    }
    (min_delta, if min_delta==0 {ply_to_leaf_min} else {ply_to_leaf_max})
}

/// Current is OR -> child is AND: sum(dn)
/// 0 -> all of the children are not tsumi
/// MAX -> one of the children leads to tsumi
/// Current is AND -> child is OR: sum(pn)
/// 0 -> all of the children are tsumi
/// MAX -> one of the children leads to not tsumi
fn sum_phi(children: &Vec<(u64, Move)>, hash_table: &mut HashTable) -> u64 {
    let mut sum = 0;
    for (hash, _) in children {
        let (p, _, _) = look_up_hash(&hash, &hash_table);
        if *p == MAX {
            // println!("Hmm going to overflow? {}", mov);
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

fn get_moves(pos: &mut Position, is_tsumi: bool, hash_table: HashTable) -> Vec<String> {
    let mut moves = Vec::new();
    get_moves_inner(pos, is_tsumi, hash_table, &mut moves);
    moves
}
fn get_moves_inner(pos: &mut Position, is_tsumi: bool, hash_table: HashTable, moves: &mut Vec<String>) {
    let attacker = pos.side_to_move() == Color::Black;
    let mut best = Option::None;
    let mut ply_to_leaf = -1;
    for (hash, mov) in generate_children(pos).0 {
        let (p, d, ply) = look_up_hash(&hash, &hash_table);
        if if is_tsumi == attacker { *d == 0 } else { *p == 0 } && ply_to_leaf<*ply {
            best = Option::Some(mov);
            ply_to_leaf = *ply;
        }
    }
    match best {
        Option::Some(best) => {
            moves.push(best.to_string());
            pos.make_move(best).unwrap();
            get_moves_inner(pos, is_tsumi, hash_table, moves);
        },
        _ => {
        }
    }
}