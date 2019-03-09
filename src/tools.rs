use num_traits::cast::{NumCast, ToPrimitive};
use std::fmt::Debug;
use std::ops::Range;

trait Mappable<F, T>
{
    fn map_interval_int(self, from: Range<F>, into:Range<T>, clamped:bool, inv:bool) -> T;
    fn map_interval(self, from: Range<F>, into:Range<T>, clamped:bool) -> T;
    fn map_interval_rev(self, from: Range<F>, into:Range<T>, clamped:bool) -> T;
}

impl<F, T> Mappable<F, T> for F
where
    F: ToPrimitive + Debug,
    T: ToPrimitive + NumCast + Debug,
{
    fn map_interval_int(self, from: Range<F>, into:Range<T>, clamped:bool, inv:bool) -> T
    {
        let mapped = {
            let from: Range<f64> = from.start.to_f64().unwrap()..from.end.to_f64().unwrap();
            let into: Range<f64> = into.start.to_f64().unwrap()..into.end.to_f64().unwrap();
            let f: f64 = self.to_f64().unwrap();

            let amplitude = (from.end - from.start) + std::f64::MIN_POSITIVE;
            let ratio = (f - from.start) / amplitude;
            let mapped = ratio * (into.end - into.start);

            let mapped = if inv {
                into.end - mapped
            } else {
                into.start + mapped
            };
            if clamped {
                mapped.max(into.start).min(into.end)
            } else {
                mapped
            }
        };
        match T::from(mapped) {
            Some(mapped) => mapped,
            None => {
                eprintln!("could not cast {:?} from {:?} to {:?}", self, from, into);
                T::from(into.start).unwrap()
            }
        }
    }
    fn map_interval(self, from: Range<F>, into:Range<T>, clamped:bool) -> T {
        self.map_interval_int(from, into, clamped, false)
    }
    fn map_interval_rev(self, from: Range<F>, into:Range<T>, clamped:bool) -> T {
        self.map_interval_int(from, into, clamped, true)
    }
}

use itertools::Itertools;
use std::cmp::PartialOrd;
use std::iter::IntoIterator;

trait Minmaxable<'a, T:'a>
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

fn normalize(data:&[f32]) -> Vec<f32> {
    let (min, max) = data.minmax();

    data.iter().map(|&f| (f - min) / (max - min)).collect_vec()
}