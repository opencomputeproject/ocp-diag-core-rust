// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

pub trait VecExt<T, U> {
    fn map_option<F>(&self, func: F) -> Option<Vec<U>>
    where
        F: Fn(&T) -> U;
}

impl<T, U> VecExt<T, U> for Vec<T> {
    fn map_option<F>(&self, func: F) -> Option<Vec<U>>
    where
        F: Fn(&T) -> U,
    {
        (!self.is_empty()).then_some(self.iter().map(func).collect())
    }
}
