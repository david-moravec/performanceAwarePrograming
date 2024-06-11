use std::f64::consts::PI;

use crate::generator::CoordinatePair;

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
