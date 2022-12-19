//! Data utilities 
use either::Either;
use itertools::Itertools;
use serde::{Serializer, Deserializer, Serialize, Deserialize};
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;
use rayon::prelude::*;

use crate::PictureResolution;


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// HIGH-CAPACITY-VEC
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

/// This data structure is a wrapper around `Vec` that preallocates memory in
/// chunks VIA the chunk size field. Which interesting, I’ve since discovered
/// that this is already the default behavior of `Vec` if it’s been initialized
/// with the `Vec::with_capacity` method. 
/// 
/// **Although**, I’m not sure if this is a platform specific optimization for
/// environments with relatively high memory capacity (such as my desktop),
/// specially, I’m not sure if this also applies to mobile devices. I could
/// test this but at the moment it seems easier to just use this wrapper which
/// will ensure consistent behavior and find out later if this data structure
/// is redundant. 
/// 
/// One benefit of keeping this around is that I may want to alter the
/// behavior at some point, such as e.g. starting with a given initial
/// capacity, and thereafter grow the vec using a different value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighCapacityVec<T> {
    data: Vec<T>,
    initial_capacity: usize,
    reallocation_capacity_chunk_size: usize,
}

impl<T> HighCapacityVec<T> {
    pub fn new(
        initial_capacity: usize,
        reallocation_capacity_chunk_size: usize,
    ) -> Self {
        let vec = Vec::with_capacity(initial_capacity);
        HighCapacityVec {data: vec, initial_capacity, reallocation_capacity_chunk_size}
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }
    pub fn into_iter(self) -> IntoIter<T> {
        self.data.into_iter()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> std::vec::Drain<T> {
        self.data.drain(range)
    }
    pub fn clear(&mut self) {
        self.data.clear()
    }
    /// Instead of calling `Vec::reserve_exact`, we instead call `vec::reserve`
    /// which may speculatively over-allocate, which may be problematic with
    /// very high initial capacity values on more memory constrained devices
    /// (I’m not sure if the defaults are different on such platforms). 
    pub fn push(&mut self, value: T) {
        let len = self.data.len();
        let capacity = self.data.capacity();
        if len == capacity {
            //
            self.data.reserve(self.reallocation_capacity_chunk_size);
        }
        self.data.push(value);
    }
}
impl<T> HighCapacityVec<T> where T: Clone + Send {
    pub fn par_filter(&mut self, f: impl Fn(&T) -> bool + Sync + Send) {
        self.data = self.data
            .clone()
            .into_par_iter()
            .filter(f)
            .collect::<Vec<_>>();
    }
}
impl<T> AsRef<[T]> for HighCapacityVec<T> {
    fn as_ref(&self) -> &[T] {
        self.data.as_ref()
    }
}
impl<T> HighCapacityVec<T> where T: Send + Sync {
    pub fn par_iter(&self) -> rayon::slice::Iter<T> {
        self.data.par_iter()
    }
}
impl<T> HighCapacityVec<T> where T: Send {
    pub fn par_iter_mut(&mut self) -> rayon::slice::IterMut<T> {
        self.data.par_iter_mut()
    }
    pub fn into_par_iter(self) -> rayon::vec::IntoIter<T> {
        self.data.into_par_iter()
    }
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// TRACKED HIGH-CAPACITY-VEC
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedHighCapacityVec<T> {
    stub: PhantomData<T>,
}


//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――
// CONVENT COW ALTERNATIVE FOR LISTS
//―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――


#[derive(Debug, Clone)]
pub enum CowCollection<'a, T> {
    SingletonRef(&'a T),
    SingletonOwned(T),
    Slice(&'a [T]),
    Vec(Vec<T>),
}

impl<'a, T> CowCollection<'a, T> {
    pub fn is_singleton_ref(&self) -> bool {match self {CowCollection::SingletonRef(_) => true, _ => false}}
    pub fn is_singleton_owned(&self) -> bool {match self {CowCollection::SingletonOwned(_) => true, _ => false}}
    pub fn is_slice(&self) -> bool {match self {CowCollection::Slice(_) => true, _ => false}}
    pub fn is_vec(&self) -> bool {match self {CowCollection::Vec(_) => true, _ => false}}
    /// Matches any vec or slice variant.
    pub fn as_slice(&self) -> Option<&[T]> {
        match self {
            CowCollection::Slice(x) => Some(x),
            CowCollection::Vec(x) => Some(x.as_slice()),
            _ => None,
        }
    }
    /// Matches any singleton variants.
    pub fn as_singleton(&self) -> Option<&T> {
        match self {
            CowCollection::SingletonRef(x) => Some(x),
            CowCollection::SingletonOwned(x) => Some(&x),
            _ => None,
        }
    }
    /// Matches all variants.
    pub fn into_vec(self) -> Vec<T>  where T:Clone {
        match self {
            CowCollection::SingletonRef(x) => vec![x.clone()],
            CowCollection::SingletonOwned(x) => vec![x],
            CowCollection::Slice(x) => x.to_vec(),
            CowCollection::Vec(x) => x,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            CowCollection::SingletonRef(_) => true,
            CowCollection::SingletonOwned(_) => true,
            CowCollection::Slice(x) => x.is_empty(),
            CowCollection::Vec(x) => x.is_empty(),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            CowCollection::SingletonRef(_) => 1,
            CowCollection::SingletonOwned(_) => 1,
            CowCollection::Slice(x) => x.len(),
            CowCollection::Vec(x) => x.len(),
        }
    }
    pub fn map_collect_vec<U>(&self, f: impl Fn(&T) -> U) -> Vec<U> {
        match self {
            CowCollection::SingletonRef(x) => vec![f(x)],
            CowCollection::SingletonOwned(x) => vec![f(x)],
            CowCollection::Slice(x) => x.iter().map(f).collect_vec(),
            CowCollection::Vec(x) => x.iter().map(f).collect_vec(),
        }
    }
    pub fn zip_collect_vec<U, V>(
        &self,
        other: &CowCollection<'_, U>,
        f: impl Fn((&T, &U)) -> V,
    ) -> Vec<V> where T: Clone, U: Clone {
        use CowCollection::{SingletonRef, SingletonOwned, Slice, Vec};
        match (self.as_singleton(), other.as_singleton()) {
            (Some(x), Some(y)) => {
                return vec![f((x, y))]
            }
            _ => {}
        }
        match (self.as_slice(), other.as_slice()) {
            (Some(x), Some(y)) => {
                return x.iter().zip(y.iter()).map(f).collect_vec();
            }
            _ => {}
        }
        let xs = self.clone().into_vec();
        let ys = other.clone().into_vec();
        xs.iter().zip(ys.iter()).map(f).collect_vec()
    }
    pub fn zip_any_satisfy<U>(
        &self,
        other: &CowCollection<'_, U>,
        f: impl Fn((&T, &U)) -> bool,
    ) -> bool where T: Clone, U: Clone {
        use CowCollection::{SingletonRef, SingletonOwned, Slice, Vec};
        match (self.as_singleton(), other.as_singleton()) {
            (Some(x), Some(y)) => {
                return f((x, y))
            }
            _ => {}
        }
        match (self.as_slice(), other.as_slice()) {
            (Some(x), Some(y)) => {
                return x.iter().zip(y.iter()).any(f);
            }
            _ => {}
        }
        let xs = self.clone().into_vec();
        let ys = other.clone().into_vec();
        xs.iter().zip(ys.iter()).any(f)
    }
    pub fn any(&self, f: impl Fn(&T) -> bool) -> bool {
        match self {
            CowCollection::SingletonRef(x) => f(x),
            CowCollection::SingletonOwned(x) => f(x),
            CowCollection::Slice(x) => x.iter().any(f),
            CowCollection::Vec(x) => x.iter().any(f),
        }
    }
    pub fn all(&self, f: impl Fn(&T) -> bool) -> bool {
        match self {
            CowCollection::SingletonRef(x) => f(x),
            CowCollection::SingletonOwned(x) => f(x),
            CowCollection::Slice(x) => x.iter().all(f),
            CowCollection::Vec(x) => x.iter().all(f),
        }
    }
}


// impl<'a, T> CowCollection<'a, T> where T: crate::frontend::DrawableObject {
//     fn to_content_ref(&self, picture_resolution: PictureResolution) -> ContentRef<'a, T> {
//         if let Some(slice) = self.as_slice() {
//             // ContentRef {
//             //     picture_resolution,
//             //     items: unimplemented!()
//             // }
//         }
//         ContentRef {
//             picture_resolution,
//             items: unimplemented!()
//         }
//     }
// }



impl<T> From<Vec<T>> for CowCollection<'_, T> {
    fn from(x: Vec<T>) -> Self {
        CowCollection::Vec(x)
    }
}
impl<'a, T> From<&'a [T]> for CowCollection<'a, T> {
    fn from(x: &'a [T]) -> Self {
        CowCollection::Slice(x)
    }
}
impl<T> From<T> for CowCollection<'_, T> {
    fn from(x: T) -> Self {
        CowCollection::SingletonOwned(x)
    }
}
impl<'a, T> From<&'a T> for CowCollection<'a, T> {
    fn from(x: &'a T) -> Self {
        CowCollection::SingletonRef(x)
    }
}

pub struct CowCollectionIntoIterator<'a, T> {
    index: usize,
    data: CowCollection<'a, T>,
}

// impl<'a, T> Iterator for CowCollectionIntoIterator<'a, T> {
//     type Item = &'a T;
//     fn next(&mut self) -> Option<&'a T> {
//         match &self.data {
//             CowCollection::SingletonRef(x) => {
//                 if self.index == 0 {
//                     self.index = self.index + 1;
//                     return Some(*x);
//                 }
//             },
//             CowCollection::SingletonOwned(x) => {
//                 if self.index == 0 {
//                     self.index = self.index + 1;
//                     return Some(x);
//                 }
//             },
//             CowCollection::Slice(x) => {

//             },
//             CowCollection::Vec(x) => {

//             },
//         }
//         // let result = match self.index {
//         //     0 => self.pixel.r,
//         //     1 => self.pixel.g,
//         //     2 => self.pixel.b,
//         //     _ => return None,
//         // };
//         // self.index += 1;
//         // Some(result)
//         unimplemented!()
//     }
// }


// impl IntoIterator for Pixel {
//     type Item = i8;
//     type IntoIter = PixelIntoIterator;

//     fn into_iter(self) -> Self::IntoIter {
//         PixelIntoIterator {
//             pixel: self,
//             index: 0,
//         }
//     }
// }

