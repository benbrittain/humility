// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::ffi::OsString;

use clap::ArgMatches;
use humility_cmd::Command;
use humility_cmd::{Args, Subcommand};

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use anyhow::Result;

mod cmd;
mod repl;


fn main() {
    run(&mut std::env::args_os());
}

pub fn run<I, T>(input: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let (commands, m, mut args) = parse_args(input);

    //
    // The only condition under which we don't require a command is if
    // --version has been specified.
    //
    if args.version {
        println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    } else if args.cmd.is_none() {
        eprintln!("humility failed: subcommand expected (--help to list)");
        std::process::exit(1);
    }

    let log_level = if args.verbose { "trace" } else { "warn" };

    let env = env_logger::Env::default().filter_or("RUST_LOG", log_level);

    env_logger::init_from_env(env);

    let mut context = humility::Context::new();


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
                log::error!("cannot specify both a dump and an archive");
                std::process::exit(1);
            }

            (false, false) => {
                log::error!(
                    "both dump and archive have been set via environment \
                    variables; unset one of them, or use a command-line option \
                    to override"
                );
                std::process::exit(1);
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

    execute_subcommand(&mut context, commands, args).unwrap();
}

pub fn parse_args<I, T>(input: I) -> (HashMap<&'static str, Command>, ArgMatches, Args)
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
    let (commands, clap) = cmd::init(Args::command());

    let input: Vec<_> = input.into_iter().collect();
    let input2 = input.clone();

    let m = clap.get_matches_from(input.into_iter());
    let _args = Args::from_arg_matches(&m);

    /*
     * If we're here, we know that our arguments pass muster from the
     * Structopt/ Clap perspective.
     */
    (commands, m, Args::parse_from(input2.into_iter()))
}

pub fn execute_subcommand(context: &mut humility::Context, commands: HashMap<&'static str, Command>, args: Args) -> Result<()> {
    //
    // This unwrap is safe -- we have checked that cmd is non-None above.
    //
    let Subcommand::Other(subargs) = args.cmd.as_ref().unwrap();

    cmd::subcommand(context, &commands, &args, subargs)?;

    Ok(())
}

#[test]
fn validate_clap() {
    let (_, clap) = cmd::init(Args::command());
    clap.clone().debug_assert();
}
