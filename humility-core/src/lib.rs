// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use hubris::HubrisArchive;

pub mod arch;
pub mod core;
pub mod hubris;

#[macro_use]
extern crate num_derive;

/// Give CLI output for the user
///
/// This macro is intended to be used whenever producing secondary output to the
/// terminal for users to see. It is its own macro for two reasons:
///
/// 1. it will prepend "humility: " to the output
/// 2. it uses stderr rather than stdout
///
/// By using this macro, if we want to change these two things, it's much easier
/// than changing every single eprintln! in the codebase.
#[macro_export]
macro_rules! msg {
    ($fmt:expr) => ({
        eprintln!(concat!("humility: ", $fmt));
    });
    ($fmt:expr, $($arg:tt)*) => ({
        eprintln!(concat!("humility: ", $fmt), $($arg)*);
    });
}

pub struct ExecutionContext {
    pub core: Option<Box<dyn core::Core>>,
    pub history: Vec<String>,
    pub archive: Option<HubrisArchive>,
}

impl ExecutionContext {
    pub fn new() -> ExecutionContext {
        ExecutionContext { 
            core: None,
            history: Vec::new(),
            archive: None,
         }
    }
}
