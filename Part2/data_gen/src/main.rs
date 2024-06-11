mod generator;
mod haversine;

use std::{fs::File, io::Write};

use clap::Parser;
use generator::{generate_answers, generate_pairs, pairs_to_str, serialize_vec};

const EARTH_RAIDUS: f64 = 6372.8;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    n: usize,

    #[arg(short, long)]
    seed: i64,

    #[arg(short, long)]
    uniform: bool,
}

fn main() {
    let args = Args::parse();
    let pairs = generate_pairs(args.n, args.seed, args.uniform);
    let answers = generate_answers(&pairs);

    let expected_sum = answers.last().cloned().unwrap();

    let filename_json: String = format!("data_{}_flex.json", args.n);
    let filename_answer: String = format!("data_{}_answer.json", args.n);

    File::create(filename_json)
        .unwrap()
        .write(&pairs_to_str(&pairs).into_bytes())
        .unwrap();

    File::create(filename_answer)
        .unwrap()
        .write(&serialize_vec(answers).into_bytes())
        .unwrap();

    println!(
        "Method: {}",
        if args.uniform { "Uniform" } else { "Cluster" }
    );
    println!("Random seed: {}", args.seed);
    println!("Pair count: {}", args.n);
    println!("Expected sum: {}", expected_sum);
}
