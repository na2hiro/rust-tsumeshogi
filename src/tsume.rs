use shogi::{Position, Color};
use shogi::bitboard::Factory as BBFactory;
use crate::dfpn::{SearchResult, dfpn_inner, HashTable, get_moves, PLY_MAX, look_up_hash};
use crate::shogi_utils::generate_children;
use crate::tsume::HenbetsuResult::{CheckedEverything, FoundTsumeWithAnotherAttack};
use std::collections::HashMap;
use std::u64::MAX;
use std::cmp::min;

/// WIP: Returns a correct tsume answer
///
/// TODO: Point out Yozume
/// TODO: Remove Mudaai
/// TODO: Choose answer which doesn't leave pieces in hand
pub fn tsume(pos: &mut Position) -> SearchResult {
    let mut hash_table: HashTable = HashMap::new();
    let mut result = dfpn_inner(pos, &mut hash_table);
    println!("dfpn done");
    if result.is_tsumi() {
        let moves = check_henbetsu(pos, &mut hash_table);
        return result.with_moves(moves);
    }

    result
}

enum HenbetsuResult {
    FoundTsumeWithAnotherAttack,
    CheckedEverything(u64)
}

fn check_henbetsu(pos: &mut Position, hash_table: &mut HashTable) -> Vec<String> {
    let mut threshold_pn = 1;
    loop {
        if pos.ply() != 1{
            println!("ply must be 1 before every henbetsu check, {}, {:?}", pos, pos.move_history());
            panic!("dame");
        }
        println!("Check henbetsu {}", threshold_pn);
        match check_henbetsu_inner(pos, hash_table, threshold_pn) {
            FoundTsumeWithAnotherAttack => {

            }
            CheckedEverything(pn_min) => {
                if pn_min == MAX {
                    break;
                }
                threshold_pn = pn_min;
            }
        };
    }
    get_moves(pos, hash_table)
}
fn check_henbetsu_inner(pos: &mut Position, hash_table: &mut HashTable, threshold_pn: u64) -> HenbetsuResult {
    if pos.side_to_move() == Color::Black {
        let mut best = Option::None;
        let mut ply_to_leaf_min = PLY_MAX;
        let mut pn_min = MAX;
        for (hash, mov) in generate_children(pos).0 {
            let (_, mut d, ply) = look_up_hash(&hash, &hash_table);
            // d is pn
            if d == 0 && *ply < ply_to_leaf_min {
                best = Option::Some(mov);
                ply_to_leaf_min = *ply;
            }
            if d == threshold_pn {
                pos.make_move(mov).unwrap();
                println!("dfpn_inner: {}", pos);
                let result = dfpn_inner(pos, hash_table);
                println!("dfpn_inner: {}", if result.is_tsumi() { "+++++++++++++++++"} else {"-------------"});
                pos.unmake_move().unwrap();
                let (_, mut d_updated, _) = look_up_hash(&hash, &hash_table);
                if d_updated == 0 {
                    println!("Found tsumi in henbetsu check {} at {}", mov, pos);
                    return FoundTsumeWithAnotherAttack;
                }
            } else if d > 0 {
                pn_min = min(pn_min, d);
            }
        }
        match best {
            Some(mov) => {
                pos.make_move(mov).unwrap();
                let result1 = check_henbetsu_inner(pos, hash_table, threshold_pn);
                pos.unmake_move().unwrap();
                match result1 {
                    CheckedEverything(pn_min_descendants) => {
                        CheckedEverything(min(pn_min, pn_min_descendants))
                    }
                    tsumi => tsumi
                }
            }
            None => {
                panic!("Impossible that there is no move for attacker in henbetsu check");
            }
        }
    } else {
        let mut best = Option::None;
        let mut ply_to_leaf_max = -1;
        for (hash, mov) in generate_children(pos).0 {
            let (p, d, ply) = look_up_hash(&hash, &hash_table);
            // p is pn. Everything should be tsumi after dfpn
            assert!(*p == 0, "Everything should be tsumi after dfpn");
            if ply_to_leaf_max <*ply {
                best = Option::Some(mov);
                ply_to_leaf_max = *ply;
            }
        }
        match best {
            Some(mov) => {
                pos.make_move(mov).unwrap();
                let result1 = check_henbetsu_inner(pos, hash_table, threshold_pn);
                pos.unmake_move().unwrap();
                result1
            }
            None => {
                // Tsumi
                CheckedEverything(MAX)
            }
        }
    }
}
