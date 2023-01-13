use std::{
    collections::{hash_map::RawEntryMut, HashMap},
    hash::Hash,
    ops::{AddAssign, Div, Mul, SubAssign},
};

pub trait Cor {
    type Key;
    type Value;

    fn cor_has(&self, k: &Self::Key, v: Self::Value) -> bool
    where
        Self::Value: Copy + Ord;

    fn cor_has_times<T: Ord>(&self, key: &Self::Key, value: Self::Value) -> T
    where
        Self::Value: Copy + Ord + Div<Output = T>;

    fn cor_has_all(&self, req: &Self) -> bool
    where
        Self::Value: Copy + Ord;

    fn cor_has_all_times<T: Ord>(&self, req: &Self, max: T) -> T
    where
        Self::Value: Copy + Ord + Div<Output = T>;

    fn cor_sub(&mut self, k: &Self::Key, v: Self::Value) -> bool
    where
        Self::Value: Ord + SubAssign;

    fn cor_sub_unchecked(&mut self, k: &Self::Key, v: Self::Value)
    where
        Self::Value: SubAssign;

    fn cor_sub_times<T: Copy + Ord + Default>(
        &mut self,
        key: &Self::Key,
        req_value: Self::Value,
        max_times: T,
    ) -> T
    where
        Self::Value:
            Copy + Eq + SubAssign + Mul<T, Output = Self::Value> + Div<Self::Value, Output = T>;

    fn cor_sub_all(&mut self, req: &Self) -> bool
    where
        Self::Value: Copy + Ord + SubAssign;

    fn cor_sub_all_unchecked(&mut self, req: &Self)
    where
        Self::Value: Copy + Ord + SubAssign;

    fn cor_sub_all_times<T: Copy + Ord>(&mut self, req: &Self, max: T) -> T
    where
        Self::Value: Copy + Ord + Mul<T, Output = Self::Value> + SubAssign + Div<Output = T>;

    fn cor_sub_all_times_unchecked<T: Copy>(&mut self, req: &Self, count: T)
    where
        Self::Value: Copy + Mul<T, Output = Self::Value> + SubAssign;

    fn cor_move_all(&mut self, dst: &mut Self, req: &Self) -> bool
    where
        Self::Value: Ord + SubAssign + AddAssign,
        Self::Key: Clone;

    fn cor_move_all_unchecked(&mut self, dst: &mut Self, req: &Self)
    where
        Self::Value: SubAssign + AddAssign,
        Self::Key: Clone;

    fn cor_move_all_times<T: Copy + Ord>(&mut self, dst: &mut Self, req: &Self, max: T) -> T
    where
        Self::Value:
            Copy + Ord + Mul<T, Output = Self::Value> + SubAssign + AddAssign + Div<Output = T>,
        Self::Key: Clone;

    fn cor_put(&mut self, key: &Self::Key, value: Self::Value)
    where
        Self::Value: AddAssign,
        Self::Key: Clone;

    fn cor_put_all(&mut self, all: &Self)
    where
        Self::Value: Copy + AddAssign,
        Self::Key: Clone;

    fn cor_put_all_times<T: Copy>(&mut self, all: &Self, times: T)
    where
        Self::Value: Copy + AddAssign + Mul<T, Output = Self::Value>,
        Self::Key: Clone;
}

impl<K: Hash + Eq, V: Default> Cor for HashMap<K, V> {
    type Key = K;
    type Value = V;

    fn cor_has(&self, req_k: &K, req_v: V) -> bool
    where
        V: Copy + Ord,
    {
        let available = match self.get(req_k) {
            Some(available) => *available,
            None => V::default(),
        };
        available >= req_v
    }

    fn cor_has_times<T: Ord>(&self, key: &Self::Key, required: Self::Value) -> T
    where
        Self::Value: Copy + Ord + Div<Output = T>,
    {
        let available = match self.get(key) {
            Some(available) => *available,
            None => V::default(),
        };
        available / required
    }

    fn cor_has_all(&self, req: &Self) -> bool
    where
        V: Copy + Ord,
    {
        for (req_k, req_v) in req.iter() {
            let available = match self.get(req_k) {
                Some(available) => *available,
                None => V::default(),
            };
            if available < *req_v {
                return false;
            }
        }
        true
    }

    fn cor_has_all_times<T: Ord>(&self, req: &Self, mut max: T) -> T
    where
        V: Copy + Ord + Div<Output = T>,
    {
        for (req_k, req_v) in req.iter() {
            let available = match self.get(req_k) {
                Some(available) => *available,
                None => V::default(),
            };
            max = max.min(available / *req_v);
        }
        max
    }

    fn cor_sub(&mut self, req_k: &Self::Key, req_v: Self::Value) -> bool
    where
        V: Ord + SubAssign,
    {
        match self.get_mut(req_k) {
            Some(available) => {
                if *available >= req_v {
                    available.sub_assign(req_v);
                    true
                } else {
                    false
                }
            }
            None => req_v <= V::default(),
        }
    }

    fn cor_sub_unchecked(&mut self, req_k: &Self::Key, req_v: Self::Value)
    where
        V: SubAssign,
    {
        self.get_mut(req_k).unwrap().sub_assign(req_v)
    }

    fn cor_sub_times<T: Copy + Ord + Default>(
        &mut self,
        key: &Self::Key,
        req_value: Self::Value,
        max_times: T,
    ) -> T
    where
        V: Copy + Eq + SubAssign + Mul<T, Output = V> + Div<V, Output = T>,
    {
        match self.get_mut(key) {
            Some(available) => {
                let times = max_times.min(*available / req_value);
                available.sub_assign(req_value * times);
                times
            }
            None => {
                if req_value == V::default() {
                    max_times
                } else {
                    T::default()
                }
            }
        }
    }

    fn cor_sub_all(&mut self, req: &Self) -> bool
    where
        V: Copy + Ord + SubAssign,
    {
        for (req_k, req_v) in req.iter() {
            match self.get_mut(req_k) {
                Some(available) => {
                    if &*available >= req_v {
                        available.sub_assign(*req_v);
                    } else {
                        return false;
                    }
                }
                None => {
                    if *req_v > V::default() {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn cor_sub_all_unchecked(&mut self, req: &Self)
    where
        V: Copy + Ord + SubAssign,
    {
        for (req_k, req_v) in req.iter() {
            self.get_mut(req_k).unwrap().sub_assign(*req_v);
        }
    }

    fn cor_sub_all_times<T: Copy + Ord>(&mut self, req: &Self, max: T) -> T
    where
        V: Copy + Ord + Mul<T, Output = V> + SubAssign + Div<Output = T>,
    {
        let available = self.cor_has_all_times(req, max);
        for (req_k, req_v) in req.iter() {
            let given_res_amount = *req_v * available;
            self.get_mut(req_k).unwrap().sub_assign(given_res_amount); // todo: unwrap may panic if (req_v=0, has[req_k]=None, default=0>=0)
        }
        available
    }

    fn cor_sub_all_times_unchecked<T: Copy>(&mut self, req: &Self, count: T)
    where
        Self::Value: Copy + Mul<T, Output = Self::Value> + SubAssign,
    {
        for (key, req_value) in req.iter() {
            let req_total = *req_value * count;
            self.get_mut(key).unwrap().sub_assign(req_total);
        }
    }

    fn cor_move_all(&mut self, _dst: &mut Self, _req: &Self) -> bool {
        todo!()
    }

    fn cor_move_all_unchecked(&mut self, _dst: &mut Self, _req: &Self) {
        todo!()
    }

    fn cor_move_all_times<T: Copy + Ord>(&mut self, dst: &mut Self, req: &Self, max: T) -> T
    where
        V: Copy + Ord + Mul<T, Output = V> + SubAssign + AddAssign + Clone + Div<Output = T>,
        Self::Key: Clone,
    {
        let available = self.cor_has_all_times(req, max);
        for (req_k, req_v) in req.iter() {
            let given_res_amount = *req_v * available;
            self.get_mut(req_k).unwrap().sub_assign(given_res_amount);
            match dst.raw_entry_mut().from_key(req_k) {
                RawEntryMut::Occupied(mut occupied) => occupied.get_mut().add_assign(*req_v),
                RawEntryMut::Vacant(vacant) => {
                    vacant.insert(req_k.clone(), *req_v);
                }
            }
        }
        available
    }

    fn cor_put(&mut self, key: &Self::Key, value: V)
    where
        V: AddAssign,
        Self::Key: Clone,
    {
        match self.raw_entry_mut().from_key(key) {
            RawEntryMut::Occupied(mut occupied) => occupied.get_mut().add_assign(value),
            RawEntryMut::Vacant(vacant) => {
                vacant.insert(key.clone(), value);
            }
        }
    }

    fn cor_put_all(&mut self, all: &Self)
    where
        Self::Value: Copy + AddAssign,
        Self::Key: Clone,
    {
        for (key, put_value) in all.iter() {
            self.cor_put(key, *put_value);
        }
    }

    fn cor_put_all_times<T: Copy>(&mut self, all: &Self, times: T)
    where
        V: Copy + AddAssign + Mul<T, Output = V>,
        Self::Key: Clone,
    {
        for (key, put_value_single) in all.iter() {
            let put_value = *put_value_single * times;
            match self.raw_entry_mut().from_key(key) {
                RawEntryMut::Occupied(mut occupied) => occupied.get_mut().add_assign(put_value),
                RawEntryMut::Vacant(vacant) => {
                    vacant.insert(key.clone(), put_value);
                }
            }
        }
    }
}
