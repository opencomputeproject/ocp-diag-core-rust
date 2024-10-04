// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output::emitters;

// TODO: will prob need some redesign
pub struct TestState {
    pub emitter: emitters::JsonEmitter,
}

impl TestState {
    pub fn new(emitter: emitters::JsonEmitter) -> TestState {
        TestState { emitter }
    }
}
