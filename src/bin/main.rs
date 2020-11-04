use tsumeshogi_lib::tsume::{tsume};
use tsumeshogi_lib::naive_df::{naive_df};

use shogi::{Position, Move};
use shogi::bitboard::Factory as BBFactory;
use std::time::Instant;
use tsumeshogi_lib::dfpn::{SearchResult, dfpn};

fn main() {
    let sfen = "9/9/7ns/8+P/9/5n2k/3G2L2/2Gbp1S2/6G2 b 3L2rbg2s2n16p 1"; // my problem without 77to?
    // let sfen = "9/9/7ns/8+P/9/5n2k/2+pG2L2/2Gbp1S2/6G2 w 3L2rbg2s2n15p 1"; // my problem
    // let sfen = "9/9/7+Bs/9/8k/5n1S1/2+pG2L2/2Gb3L1/6GN1 w N2rg2sn2l17p 1"; // branch of my problem, white's turn
    // let sfen = "8k/9/7GP/9/9/9/9/9/9 b 2r2b3g4s4n4l17p 1";
    // let sfen = "1k7/9/1GG6/9/9/9/9/9/9 b 2r2b2g4s4n4l18p 1"; // 3te with henchou
    // let sfen = "7nl/7k1/9/5P1Pp/9/8N/9/9/9 b GS2r2b3g3s2n3l15p 1"; // 5te where king needs to choose longest path
    // pos.set_sfen(std::env::args().nth(1).unwrap().as_str()).unwrap();

    solve(sfen, dfpn);
}

fn solve<F>(sfen: &str, solver: F) where F: Fn(&mut Position) -> SearchResult {
    let start = Instant::now();

    BBFactory::init();

    let mut pos = Position::new();
    pos.set_sfen(sfen).unwrap();

    let result = solver(&mut pos);
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
