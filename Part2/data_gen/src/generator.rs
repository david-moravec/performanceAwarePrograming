use core::fmt;
use rand::{rngs::ThreadRng, Rng};
use std::{iter, ops::RangeInclusive};

use crate::{haversine::reference_haversine, EARTH_RAIDUS};

const X_RANGE: RangeInclusive<f64> = -180.0..=180.0;
const Y_RANGE: RangeInclusive<f64> = -90.0..=90.0;
const CLUSTER_COUNT: usize = 16;

pub type Pair<T> = (T, T);
pub type Coordinate = Pair<f64>;
pub type CoordinatePair = Pair<Coordinate>;

fn pair(x0: f64, y0: f64, x1: f64, y1: f64) -> CoordinatePair {
    ((x0, y0), (x1, y1))
}

fn random_pair(rng: &mut ThreadRng) -> CoordinatePair {
    pair(
        rng.gen_range(X_RANGE),
        rng.gen_range(Y_RANGE),
        rng.gen_range(X_RANGE),
        rng.gen_range(Y_RANGE),
    )
}

fn random_pair_in_ranges(
    rng: &mut ThreadRng,
    x_range: RangeInclusive<f64>,
    y_range: RangeInclusive<f64>,
) -> CoordinatePair {
    pair(
        rng.gen_range(x_range.clone()),
        rng.gen_range(y_range.clone()),
        rng.gen_range(x_range.clone()),
        rng.gen_range(y_range.clone()),
    )
}

pub fn coor_pair_to_str(coord_pair: &CoordinatePair) -> String {
    let (p1, p2) = (coord_pair.0, coord_pair.1);
    format!(
        r#"{{"x0":{}, "y0":{}, "x1":{}, "y1":{}}}"#,
        p1.0, p1.1, p2.0, p2.1
    )
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

fn generate_from_range(range: RangeInclusive<f64>) -> f64 {
    rand::thread_rng().gen_range(range)
}

fn generate_x() -> f64 {
    generate_from_range(X_RANGE)
}

fn generate_y() -> f64 {
    generate_from_range(Y_RANGE)
}

pub fn generate_pairs(n: usize, uniform: bool) -> Vec<CoordinatePair> {
    if uniform {
        generate_pairs_in_range(n, X_RANGE, Y_RANGE)
    } else {
        generate_pairs_cluster(n)
    }
}

pub fn generate_pairs_in_range(
    n: usize,
    x_range: RangeInclusive<f64>,
    y_range: RangeInclusive<f64>,
) -> Vec<CoordinatePair> {
    let mut rng = rand::thread_rng();

    iter::repeat_with(|| random_pair_in_ranges(&mut rng, x_range.clone(), y_range.clone()))
        .take(n)
        .collect()
}

fn generate_pairs_cluster(n: usize) -> Vec<CoordinatePair> {
    iter::repeat_with(|| generate_random_x_y_ranges())
        .take(CLUSTER_COUNT as usize)
        .flat_map(|(x_range, y_range)| generate_pairs_in_range(n / CLUSTER_COUNT, x_range, y_range))
        .collect()
}

fn generate_random_x_y_ranges() -> Pair<RangeInclusive<f64>> {
    (
        generate_random_range_within(X_RANGE),
        generate_random_range_within(Y_RANGE),
    )
}

fn generate_random_range_within(range: RangeInclusive<f64>) -> RangeInclusive<f64> {
    let a = rand::thread_rng().gen_range(range.clone()).ceil().abs();
    -a..=a
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
    use super::*;
    use crate::generator::pairs_to_str;

    #[test]
    fn test_generate_pairs() {
        let pairs = generate_pairs(20, true);
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
