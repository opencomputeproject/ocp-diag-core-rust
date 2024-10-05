// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use crate::output::emitter;

// TODO: will prob need some redesign
pub struct TestState {
    pub emitter: emitter::JsonEmitter,
}

impl TestState {
    pub fn new(emitter: emitter::JsonEmitter) -> TestState {
        TestState { emitter }
    }
}
