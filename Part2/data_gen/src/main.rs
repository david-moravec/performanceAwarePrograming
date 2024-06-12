mod generator;
mod haversine;
mod parser;

use std::{fs::File, io::Write};

use clap::Parser;
use generator::{generate_answers, generate_pairs, pairs_to_str, serialize_vec};

use crate::{
    generator::check_answers,
    parser::{deserialize_answers_json, deserialize_json_input},
};

const EARTH_RAIDUS: f64 = 6372.8;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    n: usize,

    #[arg(short, long)]
    uniform: bool,
}

fn main() {
    let args = Args::parse();
    let pairs = generate_pairs(args.n, args.uniform);
    let answers = generate_answers(&pairs);

    let expected_sum = answers.last().cloned().unwrap();

    let filename_json: String = format!("data_{}_flex.json", args.n);
    let filename_answer: String = format!("data_{}_answer.f64", args.n);

    File::create(&filename_json)
        .unwrap()
        .write(&pairs_to_str(&pairs).into_bytes())
        .unwrap();

    File::create(&filename_answer)
        .unwrap()
        .write(&serialize_vec(answers).into_bytes())
        .unwrap();

    println!(
        "Method: {}",
        if args.uniform { "Uniform" } else { "Cluster" }
    );
    println!("Pair count: {}", args.n);
    println!("Expected sum: {}", expected_sum);

    let pairs_parsed = deserialize_json_input(File::open(filename_json).unwrap());
    let answers_parsed = deserialize_answers_json(File::open(filename_answer).unwrap());

    let computed_answers = check_answers(&pairs_parsed, &answers_parsed);

    let computed_sum = computed_answers.last().unwrap().clone();

    println!("Computed sum: {}", computed_sum);
    println!("Difference is: {:.6}", (computed_sum - expected_sum).abs());
}
