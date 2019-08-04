extern crate variant;

use rand::{thread_rng, SeedableRng, Rng};
use variant::variant::{Variant, FnVariant};
use variant::tree_rng::{TreeRng, Split};

fn main() {
    let mut rng = TreeRng::from_rng(&mut thread_rng()).unwrap();
    let [mut rng1, mut rng2]: [TreeRng; 2] = rng.split();

    let circles = FnVariant::from(|rng: &mut TreeRng| rng.gen_range(0.0, 1.0))
        .map(|e| 2.0 * e)
        .constrain(|e| *e > 1.0);
        .pool(3, |a: &f64, b: &f64| (*a - *b).ab s() > 0.2);

    for i in 0..100 {
        let val = circles.next(&mut rng1);
        let x = val;

        println!("{:?}", round(val, 32));
    }

}

fn round(x: f64, digits: i32) -> f64 {
    let int_space: f64 = x * 10.0f64.powi(digits);
    int_space.round() / 10.0f64.powi(digits)
}