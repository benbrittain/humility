// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Result, bail};
use clap::ArgMatches;
use cli::Cli;
use hubris::HubrisArchive;

pub mod arch;
pub mod core;
pub mod hubris;
pub mod cli;

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
    pub cli: Cli,
}

impl ExecutionContext {
    pub fn new(mut cli: Cli, m: &ArgMatches) -> Result<ExecutionContext> {
        //
        // Check to see if we have both a dump and an archive.  Because these
        // conflict with one another but because we allow both of them to be
        // set with an environment variable, we need to manually resolve this:
        // we want to allow an explicitly set value (that is, via the command
        // line) to win the conflict.
        //
        if cli.dump.is_some() && cli.archive.is_some() {
            match (m.occurrences_of("dump") == 1, m.occurrences_of("archive") == 1)
            {
                (true, true) => {
                    bail!("cannot specify both a dump and an archive");
                }

                (false, false) => {
                    bail!(
                        "both dump and archive have been set via environment \
                        variables; unset one of them, or use a command-line option \
                        to override"
                    );
                }

                (true, false) => {
                    log::warn!(
                        "dump on command-line overriding archive in environment"
                    );
                    cli.archive = None;
                }

                (false, true) => {
                    log::warn!(
                        "archive on command-line overriding dump in environment"
                    );
                    cli.dump = None;
                }
            }
        }

        Ok(ExecutionContext { 
            core: None,
            history: Vec::new(),
            archive: None,
            cli,
        })
    }
}
