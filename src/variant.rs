use rand::{thread_rng, Rng, SeedableRng};
use rand::distributions::{Distribution, Uniform};
use std::marker::PhantomData;
use std::iter;

pub struct VariantIter<'r, V, R> {
    variant: V,
    rng: &'r mut R
}

impl<'r, V: Variant> Iterator for VariantIter<'r, V, V::Rng>  {
    type Item = V::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.variant.next(self.rng))
    }
}

//Variant::from_fn(|rng: TreeRng| rng.choose(something))
// .map(|e| 2 * e)
// .pool(20, |a,b| (a-b).abs() > 10.0)
// .next(&mut thread_rng());

pub trait Variant: Sized {
    type Item;
    type Rng: Rng;

    fn next(&self, rng: &mut Self::Rng) -> Self::Item;

    fn from_fn<F, R: Rng>(function: F) -> FnVariant<F, R, Self::Item> where F: Fn(&mut R) -> Self::Item {
        FnVariant {
            function,
            _rng: PhantomData,
            _type: PhantomData
        }
    }

//    fn from_fn<R: Rng, F>(function: F) -> FnVariant<F, R, Self::Item> where F: Fn(&mut R) -> Self::Item {
//        FnVariant::new(function)
//    }

    fn from_distribution<R: Rng, D>(distribution: D) -> DistributionVariant<D, Self::Item, Self::Rng> where D: Distribution<Self::Item> {
        DistributionVariant::from(distribution)
    }

    fn into_iter<R: Rng>(self, rng: &mut R) -> VariantIter<Self, R> {
        VariantIter {
            variant: self,
            rng
        }
    }

    fn merge(self, weight: f64) -> MergeVariant<Self, ()> {
        MergeVariant {
            next: None,
            variant: self,
            threshold: 0.0,
            total_weight: weight
        }
    }

    fn map<M, A, B>(self, map: M) -> MapVariant<M, Self> where M: Fn(A) -> B {
        MapVariant {
            map,
            variant: self
        }
    }

    fn constrain<F>(self, constraint: F) -> ConstrainedVariant<F, Self> where F: Fn(&Self::Item) -> bool {
        ConstrainedVariant {
            constraint,
            variant: self
        }
    }

    fn try_constrain<F>(self, constraint: F, attempts: usize) -> TryConstrainedVariant<F, Self> where F: Fn(&Self::Item) -> bool {
        TryConstrainedVariant {
            constraint,
            variant: self,
            attempts: attempts
        }
    }

    fn pool<F>(self, size: usize, self_constraint: F) -> PoolVariant<F, Self> where F: Fn(&Self::Item, &Self::Item) -> bool {
        PoolVariant {
            self_constraint,
            variant: self,
            size
        }
    }

    fn try_pool<F>(self, size: usize, self_constraint: F, attempts: usize) -> TryPoolVariant<F, Self> where F: Fn(&Self::Item, &Self::Item) -> bool {
        TryPoolVariant {
            self_constraint,
            variant: self,
            size,
            attempts
        }
    }
}

pub struct MapVariant<M, V> {
    map: M,
    variant: V
}

impl<M, V: Variant, B> Variant for MapVariant<M, V> where M: Fn(V::Item) -> B {
    type Item = B;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> Self::Item {
        (&self.map)(self.variant.next(rng))
    }
}

pub trait VariantChain: Variant {
    type State;

    fn next_chain(&self, chain_state: Self::State, rng: &mut Self::Rng) -> Self::Item;
}

pub struct MergeVariant<V, C> {
    next: Option<C>,
    variant: V,
    threshold: f64,
    total_weight: f64
}

type MergeVariantState = f64;
impl<V, C, T> MergeVariant<V, C>
    where V: Variant<Item=T>
{
    pub fn with<V2>(self, variant: V2, weight: f64) -> MergeVariant<V2, Self>
        where V2: Variant<Item=T>
    {
        MergeVariant {
            threshold: self.threshold + weight,
            total_weight: self.total_weight + weight,
            next: Some(self),
            variant
        }
    }
}

impl<V, C, T, R: Rng> VariantChain for MergeVariant<V, C>
    where V: Variant<Item=T, Rng=R>,
          C: Variant<Item=T, Rng=R> + VariantChain<State=MergeVariantState>
{
    type State = MergeVariantState;

    fn next_chain(&self, state: MergeVariantState, rng: &mut R) -> T {
        if state > self.threshold {
            self.variant.next(rng)
        } else {
            match &self.next {
                Some(next) => next.next_chain(state, rng),
                None => panic!("Should never fall through last choice in Variant Chain")
            }
        }
    }
}

impl<V, T> VariantChain for MergeVariant<V, ()>
    where V: Variant<Item=T>
{
    type State = MergeVariantState;

    fn next_chain(&self, state: Self::State, rng: &mut Self::Rng) -> Self::Item {
        if state > self.threshold {
            self.variant.next(rng)
        } else {
            panic!("Fork variant should always be selected");
        }
    }
}

impl<V, C, T, R: Rng> Variant for MergeVariant<V, C>
    where V: Variant<Item=T, Rng=R>,
          C: Variant<Item=T, Rng=R> + VariantChain<State=MergeVariantState>,
{
    type Item = T;
    type Rng = V::Rng;

    fn next(&self, rng: &mut Self::Rng) -> Self::Item {
        let choice = rng.gen_range(0f64, self.total_weight);
        self.next_chain(choice, rng)
    }
}

impl<V, T> Variant for MergeVariant<V, ()>
    where V: Variant<Item=T>
{
    type Item = T;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> Self::Item {
        self.variant.next(rng)
    }
}

impl<D: Distribution<T>, T, R: Rng> From<D> for DistributionVariant<D, T, R> {
    fn from(distribution: D) -> DistributionVariant<D, T, R> {
        DistributionVariant {
            distribution,
            _item: PhantomData,
            _rng: PhantomData
        }
    }
}

pub struct DistributionVariant<D, T, R> {
    distribution: D,
    _item: PhantomData<T>,
    _rng: PhantomData<R>
}

impl<D, T, R: Rng> Variant for DistributionVariant<D, T, R> where D: Distribution<T> {
    type Item = T;
    type Rng = R;

    fn next(&self, rng: &mut R) -> Self::Item {
        self.distribution.sample(rng)
    }
}

pub struct ConstrainedVariant<C, V: Variant> {
    constraint: C,
    variant: V
}

pub struct TryConstrainedVariant<C, V: Variant> {
    constraint: C,
    variant: V,
    attempts: usize
}

impl<C, V: Variant> Variant for ConstrainedVariant<C, V> where C: Fn(&V::Item) -> bool {
    type Item = V::Item;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> V::Item
    {
        let c = &self.constraint;

        iter::repeat(())
            .map(|e| self.variant.next(rng))
            .filter(|e| c(&e))
            .next()
            .expect("Should have generated a value")
    }
}

impl<C, V: Variant> Variant for TryConstrainedVariant<C, V> where C: Fn(&V::Item) -> bool {
    type Item = Option<V::Item>;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> Option<V::Item> {
        let c = &self.constraint;

        let mut val = self.variant.next(rng);
        let mut tries = 1;

        while !c(&val) {
            if tries > self.attempts {
                return None;
            }

            val = self.variant.next(rng);
            tries += 1;
        }

        Some(val)
    }
}

pub struct PoolVariant<SC, V: Variant> {
    self_constraint: SC,
    variant: V,
    size: usize
}

pub struct TryPoolVariant<SC, V: Variant> {
    self_constraint: SC,
    variant: V,
    size: usize,
    attempts: usize
}

impl<SC, V: Variant> Variant for PoolVariant<SC, V> where SC: Fn(&V::Item, &V::Item) -> bool {
    type Item = Vec<V::Item>;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> Self::Item {
        let mut ret = Vec::with_capacity(self.size);

        while ret.len() < self.size {
            let val = self.variant.next(rng);
            if ret.iter().all(|e| (self.self_constraint)(e, &val)) {
                ret.push(val);
            }
        }

        ret
    }
}

impl<SC, V: Variant> Variant for TryPoolVariant<SC, V> where SC: Fn(&V::Item, &V::Item) -> bool {
    type Item = Option<Vec<V::Item>>;
    type Rng = V::Rng;

    fn next(&self, rng: &mut V::Rng) -> Self::Item {
        let mut ret = Vec::with_capacity(self.size);
        let mut tries = 0;

        while ret.len() < self.size {
            if tries > self.attempts {
                return None;
            }

            let val = self.variant.next(rng);
            if ret.iter().all(|e| (self.self_constraint)(e, &val)) {
                ret.push(val)
            }

            tries += 1;
        }

        Some(ret)
    }
}

//struct FnVariant<F: Fn(&mut impl Rng) -> T, R: Rng, T> {
//    function: F,
//    r: PhantomData<R>,
//    t: PhantomData<T>
//}
//
//impl<F: Fn(&mut impl Rng) -> T, R: Rng, T> FnVariant<F, T> {
//    pub fn new(function: F) -> FnVariant<F, T> {
//        FnVariant {
//            function,
//            r: PhantomData,
//            t: PhantomData
//        }
//    }
//}
//
//impl<F: Fn(&mut R) -> T, T, R: Rng> Variant for FnVariant<F, R, T> {
//    type Item = T;
//
//    fn next<R2>(&self, rng: &mut R2) -> Option<Self::Item> {
//        let f = &self.function;
//        Some(f(rng))
//    }
//}

pub struct FnVariant<F: Fn(&mut R) -> T, R: Rng, T> {
    function: F,
    _rng: PhantomData<R>,
    _type: PhantomData<T>
}

impl<F: Fn(&mut R) -> T, R: Rng, T> FnVariant<F, R, T> {
    pub fn from(function: F) -> FnVariant<F, R, T> {
        FnVariant {
            function,
            _rng: PhantomData,
            _type: PhantomData
        }
    }
}

impl<F: Fn(&mut R) -> T, T, R: Rng> Variant for FnVariant<F, R, T> {
    type Item = T;
    type Rng = R;

    fn next(&self, rng: &mut R) -> Self::Item {
        let f = &self.function;
        f(rng)
    }
}


//TODO: binary exponential BoundaryGeometry - one number for each combination of inside/outside

