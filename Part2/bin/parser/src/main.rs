use clap::Parser;
use std::collections::HashMap;
use std::{fs::File, io::Read};

use util::haversine::*;

#[derive(Debug, Parser)]
struct Args {
    file_path_json: String,
    file_path_answer: String,
}

fn main() {
    let args = Args::parse();

    let pairs_parsed = deserialize_json_input(File::open(args.file_path_json).unwrap());
    let answers_parsed = deserialize_answers_json(File::open(args.file_path_answer).unwrap());
    let computed_answers = check_answers(&pairs_parsed, &answers_parsed);
    let computed_sum = computed_answers.last().unwrap().clone();
    let expected_sum = answers_parsed.last().unwrap().clone();

    println!("Computed sum: {}", computed_sum);
    println!("Difference is: {:.6}", (computed_sum - expected_sum).abs());
}

fn deserialize_json_input(mut f: File) -> Vec<CoordinatePair> {
    let mut data = vec![];
    f.read_to_end(&mut data).unwrap();
    coordinate_pairs_from_json(String::from_utf8(data).unwrap())
}

fn deserialize_answers_json(mut f: File) -> Vec<f64> {
    let mut data = vec![];
    f.read_to_end(&mut data).unwrap();
    answers_from_json(String::from_utf8(data).unwrap())
}

fn lex_json_input(mut s: String) -> Vec<HashMap<String, f64>> {
    s.retain(|c| !c.is_whitespace());
    s.drain(..s.find("[").unwrap() + 1);

    let mut result = vec![];

    while let Some(c) = s.chars().next() {
        match c {
            '{' => result.push(deserialize_hashmap(chop_string_at(&mut s, '}'))),
            ',' => s = s[1..].to_string(), // skip
            ']' => s = s[1..].to_string(), // skip
            '}' => s = s[1..].to_string(), // skip
            _ => panic!("Unknown character {}", c),
        };
    }

    result
}

fn answers_from_json(s: String) -> Vec<f64> {
    s.split('\n').map(|s| s.parse().unwrap()).collect()
}

fn coordinate_pair_from_json(json: &HashMap<String, f64>) -> CoordinatePair {
    (
        (*json.get("x0").unwrap(), *json.get("y0").unwrap()),
        (*json.get("x1").unwrap(), *json.get("y1").unwrap()),
    )
}

fn coordinate_pairs_from_json(s: String) -> Vec<CoordinatePair> {
    lex_json_input(s)
        .iter()
        .map(|json| coordinate_pair_from_json(json))
        .collect()
}

fn chop_string_at(s: &mut String, c: char) -> String {
    let mut result: String;

    if let Some(i) = s[1..].find(c) {
        result = s[1..i + 1].to_string();
        s.drain(..i + 2);
    } else {
        result = s.clone();
        result.remove(0);
        result.pop();
        s.clear();
    }

    result
}

fn deserialize_hashmap(mut json_str: String) -> HashMap<String, f64> {
    let mut result: HashMap<String, f64> = HashMap::new();

    while let Some(c) = json_str.chars().next() {
        match c {
            '"' => {
                let key: String = chop_string_at(&mut json_str, '"');
                let val_str = chop_string_at(&mut json_str, ',');
                let val: f64 = val_str
                    .parse()
                    .expect(format!("Cannot parse {} to float", val_str).as_str());
                result.insert(key, val);
            }
            '{' => json_str = json_str[1..].to_string(), // skip
            _ => panic!("Unknown character {}", c),
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chop_string_at() {
        let mut s = r#""pairs": ["a": 4]"#.to_string();

        assert_eq!("pairs", chop_string_at(&mut s, '"'));
        assert_eq!(r#": ["a": 4]"#.to_string(), s)
    }

    #[test]
    fn test_lex_answers_input() {
        let input = "1.1
2.2
3.3
4.45";

        assert_eq!(answers_from_json(input.to_string()).len(), 4);
    }

    #[test]
    fn test_desirialize_hashmap() {
        let json = r#"{"x0":102.1633205722960440,"y0":-24.9977499718717624,"x1":-14.3322557404258362,"y1":62.6708294856625940}"#;

        let map = deserialize_hashmap(json.to_string());

        assert_eq!(*map.get("x0").unwrap(), 102.1633205722960440 as f64);
        assert_eq!(*map.get("y0").unwrap(), -24.9977499718717624 as f64);
        assert_eq!(*map.get("x1").unwrap(), -14.3322557404258362 as f64);
        assert_eq!(*map.get("y1").unwrap(), 62.6708294856625940 as f64);
    }

    #[test]
    fn test_lex_json_input() {
        let pairs_str = r#"{"pairs":[
{"x0":1.1, "y0":2.1, "x1":3.1, "y1":4.1},
{"x0":5.1, "y0":6.1, "x1":7.1, "y1":8.1}
]}"#;
        let vec_map = lex_json_input(pairs_str.to_string());

        assert_eq!(vec_map.len(), 2)
    }
}
