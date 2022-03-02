// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::ffi::OsString;

use anyhow::bail;
use clap::ArgMatches;
use humility_cmd::Command;
use humility_cmd::Cli;

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use anyhow::Result;

mod cmd;
mod repl;


fn main() -> Result<()> {
    let (commands, m, mut args) = parse_args(&mut std::env::args_os());

    //
    // The only condition under which we don't require a command is if
    // --version has been specified.
    //
    if args.version {
        println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    } else if args.cmd.is_none() {
        bail!("humility failed: subcommand expected (--help to list)");
    }

    let log_level = if args.verbose { "trace" } else { "warn" };

    let env = env_logger::Env::default().filter_or("RUST_LOG", log_level);

    env_logger::init_from_env(env);

    let mut context = humility::ExecutionContext::new();

    //
    // Check to see if we have both a dump and an archive.  Because these
    // conflict with one another but because we allow both of them to be
    // set with an environment variable, we need to manually resolve this:
    // we want to allow an explicitly set value (that is, via the command
    // line) to win the conflict.
    //
    if args.dump.is_some() && args.archive.is_some() {
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
                args.archive = None;
            }

            (false, true) => {
                log::warn!(
                    "archive on command-line overriding dump in environment"
                );
                args.dump = None;
            }
        }
    }

    cmd::subcommand(&mut context, &commands, &args)
}

pub fn parse_args<I, T>(input: I) -> (HashMap<&'static str, Command>, ArgMatches, Cli)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    /*
     * This isn't hugely efficient, but we actually parse our arguments
     * twice: the first is with our subcommands grafted into our
     * arguments to get us a unified help and error message in the event
     * of any parsing value or request for a help message; if that works,
     * we parse our arguments again but relying on the
     * external_subcommand to directive to allow our subcommand to do any
     * parsing on its own.
     */
    let (commands, clap) = cmd::init(Cli::command());

    let input: Vec<_> = input.into_iter().collect();
    let input2 = input.clone();

    let m = clap.get_matches_from(input.into_iter());
    let _args = Cli::from_arg_matches(&m);

    /*
     * If we're here, we know that our arguments pass muster from the
     * Structopt/ Clap perspective.
     */
    (commands, m, Cli::parse_from(input2.into_iter()))
}

#[test]
fn validate_clap() {
    let (_, clap) = cmd::init(Cli::command());
    clap.clone().debug_assert();
}
