use num_traits::cast::{NumCast, ToPrimitive};
use std::fmt::Debug;
use std::ops::RangeInclusive as RanInc;

pub trait Mappable<F, T>
{
    fn map_interval_int(self, from: RanInc<F>, into:RanInc<T>, inv:bool) -> T;
    fn map_interval(self, from: RanInc<F>, into:RanInc<T>) -> T;
    fn map_interval_rev(self, from: RanInc<F>, into:RanInc<T>) -> T;
}

impl<F, T> Mappable<F, T> for F
where
    F: ToPrimitive + Debug,
    T: ToPrimitive + NumCast + Debug + Copy,
{
    fn map_interval_int(self, from: RanInc<F>, into:RanInc<T>, inv:bool) -> T
    {
        let mapped = {
            let from: RanInc<f64> = from.start().to_f64().unwrap() ..= from.end().to_f64().unwrap();
            let into: RanInc<f64> = into.start().to_f64().unwrap() ..= into.end().to_f64().unwrap();
            let f: f64 = self.to_f64().unwrap();

            assert!(from.start() <= from.end());
            assert!(into.start() <= into.end());
            assert!(f >= *from.start() && f <= *from.end());

            let amplitude = (from.end() - from.start()) + std::f64::MIN_POSITIVE;
            let ratio = (f - from.start()) / amplitude;
            let mapped = ratio * (into.end() - into.start());

            let mapped = if inv {
                into.end() - mapped
            } else {
                into.start() + mapped
            };
            assert!(mapped >= *into.start() && mapped <= *into.end());
            mapped
        };
        match T::from(mapped) {
            Some(mapped) => mapped,
            None => {
                eprintln!("could not cast {:?} from {:?} to {:?}", self, from, into);
                *into.start()
            }
        }
    }
    fn map_interval(self, from: RanInc<F>, into:RanInc<T>) -> T {
        self.map_interval_int(from, into, false)
    }
    fn map_interval_rev(self, from: RanInc<F>, into:RanInc<T>) -> T {
        self.map_interval_int(from, into, true)
    }
}

// Makes extracting minmax values fron an array easier

use itertools::Itertools;
use std::cmp::PartialOrd;
use std::iter::IntoIterator;

pub trait Minmaxable<'a, T:'a>
{
    fn minmax(self) -> (T, T);
}

impl<'a, T:'a, I> Minmaxable<'a, T> for I
where
    T:PartialOrd+Copy+Sized,
    I:IntoIterator<Item = &'a T>,
{
    fn minmax(self) -> (T, T)
    {
        self.into_iter().cloned().minmax().into_option().unwrap()
    }
}

// Allow for the normalization of floating point arrays
pub trait Normalizable {
    // Map the array from the range min..=max to 0..=1
    fn normalize(&mut self);
}

impl Normalizable for [f32] {
    fn normalize(&mut self) {
        let (min, max) = self.minmax();

        self.iter_mut().for_each(|i| {
            *i = (*i - min) / (max - min);
        });
    }
}

impl Normalizable for [f64] {
    fn normalize(&mut self) {
        let (min, max) = self.minmax();

        self.iter_mut().for_each(|i| {
            *i = (*i - min) / (max - min);
        });
    }
}