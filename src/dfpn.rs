extern crate wasm_bindgen;

use shogi::{Position, Move, Color};
use shogi::color::Color::Black;
use shogi::bitboard::Factory as BBFactory;
use crate::shogi_utils::{generate_children, calculate_hash};
use std::collections::HashMap;
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
    pub fn is_tsumi(&self) -> bool {
        self.is_tsumi
    }
    pub fn nodes(&self) -> u64 {
        self.nodes
    }
    pub fn moves(&self) -> &Vec<String> {
        &self.moves
    }
    pub fn nodes_incl_temporary(&self) -> u64 {
        self.nodes_incl_temporary
    }
    pub fn with_moves(&self, moves: Vec<String>) -> SearchResult {
        SearchResult {
            is_tsumi: self.is_tsumi,
            moves,
            ply_to_leaf: self.ply_to_leaf,
            nodes: self.nodes,
            nodes_incl_temporary: self.nodes_incl_temporary
        }
    }
}

type PlyToLeaf = i16;
pub const PLY_MAX: PlyToLeaf = 32767;
pub type HashTable = HashMap<u64, (u64, u64, PlyToLeaf)>;

pub fn dfpn(pos: &mut Position) -> SearchResult {
    let mut hash_table: HashTable = HashMap::new();
    let mut result = dfpn_inner(pos, &mut hash_table);
    if result.is_tsumi {
        result.moves = get_moves(pos, &hash_table);
    }
    result
}

pub fn dfpn_inner(pos: &mut Position, hash_table: &mut HashTable) -> SearchResult {
    let attacker = pos.side_to_move() == Black;
    let (mut p, mut d, mut ply_to_leaf, mut cnt, mut cnt_tmp) = mid(pos, MAX - 1, MAX - 1, hash_table);
    if p != MAX && d != MAX {
        println!("second time");
        let (_, d2, ply_to_leaf2, cnt2, cnt_tmp2) = mid(pos, p, d, hash_table);
        d=d2;
        ply_to_leaf = ply_to_leaf2;
        cnt+=cnt2;
        cnt_tmp+= cnt_tmp2;
    }

    let is_tsumi = if attacker { d == MAX } else { d == 0 };
    SearchResult {
        is_tsumi,
        moves: Vec::new(),
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

    let (children, mut node_count_incl_temporary) = generate_children(pos);

    // println!("{}, {:?}", pos, children);

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
        match pos.make_move(*best_move) {
            Ok(_) => {}
            Err(e) => {panic!("move failure {} {}, {}, {}", e, best_move, pos, pos.to_sfen())}
        };
        let (_, _, _, cnt, cnt_incl_temporary) = mid(pos, phi_n_c, delta_n_c, hash_table);
        node_count += cnt;
        node_count_incl_temporary += cnt_incl_temporary;
        match pos.unmake_move() {
            Ok(_) => {},
            Err(_) => panic!("{}, {:?}", pos, pos.move_history().last().unwrap()),
        }
    }
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

pub fn look_up_hash<'a>(hash: &u64, hash_table: &'a HashTable) -> &'a (u64, u64, PlyToLeaf) {
    hash_table.get(hash).unwrap_or(&(1u64, 1u64, 0))
}

pub fn put_in_hash(hash: u64, phi: u64, delta: u64, ply_to_leaf: PlyToLeaf, hash_table: &mut HashTable) {
    hash_table.insert(hash, (phi, delta, ply_to_leaf));
}

/// Calculates minimum of delta of children
///
/// * Current is OR -> child is AND: min(pn)
///   * 0 -> one of the children leads to tsumi
///   * MAX -> all of the children are not tsumi
/// * Current is AND -> child is OR: min(dn)
///   * 0 -> one of the children leads to not tsumi
///   * MAX -> all of the children are tsumi
fn min_delta(children: &Vec<(u64, Move)>, hash_table: &mut HashTable) -> (u64, PlyToLeaf) {
    let mut min_delta = MAX;
    let mut ply_to_leaf_min = PLY_MAX;
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

/// Calculates sum of phi of children
///
/// * Current is OR -> child is AND: sum(dn)
///   * 0 -> all of the children are not tsumi
///   * MAX -> one of the children leads to tsumi
/// * Current is AND -> child is OR: sum(pn)
///   * 0 -> all of the children are tsumi
///   * MAX -> one of the children leads to not tsumi
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

pub fn get_moves(pos: &mut Position, hash_table: &HashTable) -> Vec<String> {
    let mut moves = Vec::new();
    get_moves_inner(pos, hash_table, &mut moves);
    moves
}
fn get_moves_inner(pos: &mut Position, hash_table: &HashTable, moves: &mut Vec<String>) {
    let attacker = pos.side_to_move() == Color::Black;
    let mut best = Option::None;
    let mut ply_to_leaf = -1;
    for (hash, mov) in generate_children(pos).0 {
        let (p, d, ply) = look_up_hash(&hash, &hash_table);
        if if attacker { *d == 0 } else { *p == 0 } && ply_to_leaf<*ply {
            best = Option::Some(mov);
            ply_to_leaf = *ply;
        }
    }
    match best {
        Option::Some(best) => {
            moves.push(best.to_string());
            pos.make_move(best).unwrap();
            get_moves_inner(pos, hash_table, moves);
            pos.unmake_move().unwrap();
        },
        _ => {
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfpn() {
        BBFactory::init();

        // let sfen_no_tsumi = "8l/6s2/7kn/4G1pB1/8p/7R1/9/9/9 b Br3g3s3n3l16p 1";
        let sfen = "8l/6s2/4+P2kn/6pB1/8p/7R1/9/9/9 b Br4g3s3n3l15p 1";
        let mut pos = Position::new();
        pos.set_sfen(sfen).unwrap();
        assert_eq!(true, dfpn(&mut pos).is_tsumi);

        let sfen = "8l/6s2/4S2kn/6pB1/8p/7R1/9/9/9 b Br4g2s3n3l16p 1";
        let mut pos = Position::new();
        pos.set_sfen(sfen).unwrap();
        let result = dfpn(&mut pos);
        assert_eq!(true, result.is_tsumi);

        let sfen = "7nl/5+RBk1/9/6+r2/7pP/9/9/9/9 b Lb4g4s3n2l16p 1";
        let mut pos = Position::new();
        pos.set_sfen(sfen).unwrap();
        let result = dfpn(&mut pos);
        assert_eq!(5, result.moves.len());
        assert_eq!(true, result.is_tsumi);

    }
}