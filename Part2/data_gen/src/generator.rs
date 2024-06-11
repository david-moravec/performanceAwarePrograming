use core::fmt;
use rand::Rng;
use std::ops::RangeInclusive;

use crate::{haversine::reference_haversine, EARTH_RAIDUS};

const X_RANGE: RangeInclusive<f64> = -180.0..=180.0;
const Y_RANGE: RangeInclusive<f64> = -90.0..=90.0;

pub type Pair<T> = (T, T);
pub type Coordinate = Pair<f64>;
pub type CoordinatePair = Pair<Coordinate>;

pub fn coor_pair_to_str(coord_pair: &CoordinatePair) -> String {
    let (p1, p2) = (coord_pair.0, coord_pair.1);
    format!(
        r#"{{"x0":{}, "y0":{}, "x1":{}, "y1":{}}}"#,
        p1.0, p1.1, p2.0, p2.1
    )
}

fn generate_pair(_seed: i64, _uniform: bool) -> CoordinatePair {
    ((generate_x(), generate_y()), (generate_x(), generate_y()))
}

pub fn generate_answers(pairs: &Vec<CoordinatePair>) -> Vec<f64> {
    let mut v = Vec::from_iter(
        pairs
            .into_iter()
            .map(|ref p| reference_haversine(p, EARTH_RAIDUS)),
    );

    let avg_v = v.iter().fold(0.0, |a, e| a + e) / v.len() as f64;

    v.push(avg_v);

    v
}

pub fn serialize_vec<T>(v: Vec<T>) -> String
where
    T: fmt::Display,
{
    v.into_iter()
        .map(|e| format!("{}", e))
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_x() -> f64 {
    rand::thread_rng().gen_range(X_RANGE)
}

fn generate_y() -> f64 {
    rand::thread_rng().gen_range(Y_RANGE)
}

pub fn generate_pairs(n: usize, seed: i64, uniform: bool) -> Vec<CoordinatePair> {
    let mut result: Vec<CoordinatePair> = vec![];

    for _ in 0..n {
        result.push(generate_pair(seed, uniform));
    }

    result
}

pub fn pairs_to_str(pairs: &Vec<CoordinatePair>) -> String {
    let pairs_str: String = pairs
        .iter()
        .map(|p| coor_pair_to_str(p))
        .collect::<Vec<String>>()
        .join(",\n");

    format!("{{\"pairs\":[\n{}\n]}}", pairs_str)
}

#[cfg(test)]
mod test {
    use crate::generator::pairs_to_str;

    use super::{coor_pair_to_str, generate_pairs, CoordinatePair};

    fn pair(x0: f64, y0: f64, x1: f64, y1: f64) -> CoordinatePair {
        ((x0, y0), (x1, y1))
    }

    #[test]
    fn test_generate_pairs() {
        let pairs = generate_pairs(20, 8, true);
        assert_eq!(pairs.len(), 20)
    }

    #[test]
    fn test_coor_pair_to_str() {
        let pair: CoordinatePair = pair(1.0, 2.0, 3.0, 4.0);
        println!("{}", coor_pair_to_str(&pair));
        assert_eq!(
            coor_pair_to_str(&pair),
            r#"{"x0":1, "y0":2, "x1":3, "y1":4}"#
        )
    }

    #[test]
    fn test_pairs_to_str() {
        let pairs = vec![pair(1.1, 2.1, 3.1, 4.1), pair(5.1, 6.1, 7.1, 8.1)];
        let pairs_str = r#"{"pairs":[
{"x0":1.1, "y0":2.1, "x1":3.1, "y1":4.1},
{"x0":5.1, "y0":6.1, "x1":7.1, "y1":8.1}
]}"#;

        assert_eq!(pairs_to_str(&pairs), pairs_str)
    }
}
