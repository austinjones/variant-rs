use rand_xoshiro::Xoshiro256StarStar;
use rand::{RngCore, SeedableRng, Rng};
use array_ext::Array;

pub struct TreeRng {
    rng: Xoshiro256StarStar
}

impl TreeRng {

}

impl RngCore for TreeRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl SeedableRng for TreeRng {
    type Seed = [u8; 32];

    fn from_seed(seed: [u8; 32]) -> Self {
        TreeRng {
            rng: Xoshiro256StarStar::from_seed(seed)
        }
    }
}

pub trait Split<T> {
    fn split(self) -> T;
}


impl<T: Array<TreeRng>> Split<T> for TreeRng {
    fn split(mut self) -> T {
        // we need to create stable seeds for each child RNG.
        let iter = std::iter::repeat(()).map(|_| {
            // jump, so the child RNGs will see state which is far apart
            self.rng.jump();

            // get the seed
            let mut seed: [u8; 32] = [0u8; 32];
            self.fill_bytes(&mut seed);

            // create the child Xoshiro RNG
            TreeRng::from_seed(seed)
        });

        T::from_iter(iter).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {

    }
}