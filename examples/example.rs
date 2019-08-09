extern crate variant;

use rand::{thread_rng, SeedableRng, Rng};
use variant::{Variant, Variants};
use variant::tree_rng::{TreeRng, Split};

fn main() {
    let mut rng = TreeRng::from_rng(&mut thread_rng()).unwrap();
    let [mut rng1, mut rng2]: [TreeRng; 2] = rng.split();

    let circles = Variants::from_fn(|rng: &mut TreeRng| rng.gen_range(0.0, 1.0))
        .map(|rng, e| 2.0 * e)
        .constrain(|e| *e > 1.0)
        .pool(3, |a: &f64, b: &f64| (*a - *b).abs() > 0.2);

    for i in 0..100 {
        let val = circles.next(&mut rng1);
        println!("{:?}", val);
    }
}