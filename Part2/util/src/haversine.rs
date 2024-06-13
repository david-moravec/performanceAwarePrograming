use std::f64::consts::PI;

pub type Pair<T> = (T, T);
pub type Coordinate = Pair<f64>;
pub type CoordinatePair = Pair<Coordinate>;

const EARTH_RAIDUS: f64 = 6372.8;

fn rad_from_deg(degrees: f64) -> f64 {
    degrees / 180.0 * PI
}

pub fn reference_haversine(coord_pair: &CoordinatePair, radius: f64) -> f64 {
    let dlon = rad_from_deg(coord_pair.1 .0 - coord_pair.0 .0);
    let dlat = rad_from_deg(coord_pair.1 .1 - coord_pair.0 .1);
    let lat0 = rad_from_deg(coord_pair.0 .1);
    let lat1 = rad_from_deg(coord_pair.1 .1);

    let a = (dlat / 2.0).sin().powi(2) + lat0.cos() * lat1.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    radius * c
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

pub fn check_answers(pairs: &Vec<CoordinatePair>, answers: &Vec<f64>) -> Vec<f64> {
    let answers_new = generate_answers(pairs);

    for (i, (a_new, a)) in answers_new[..answers_new.len() - 1]
        .iter()
        .zip(answers[..answers.len() - 1].iter())
        .enumerate()
    {
        if a_new - a >= 1e-6 {
            println!(
                "Answers do not agree\nExpected: {}\nComputed: {}\nPair: {:?}",
                a, a_new, pairs[i]
            )
        }
    }

    answers_new
}
