// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{bail, Context, Result};
use clap::Command as ClapCommand;
use humility::hubris::*;
use humility::cli::Subcommand;
use humility_cmd::{Archive, Command};
use std::collections::HashMap;

//
// Our build.rs creates cmds.rs, which looks at our workspace to assemble
// the commands, and creates a function (`dcmds`) that we call to get
// a vector of them.
//
include!(concat!(env!("OUT_DIR"), "/cmds.rs"));

use crate::repl;

pub fn init(
    command: ClapCommand<'static>,
) -> (HashMap<&'static str, Command>, ClapCommand<'static>) {
    let mut cmds = HashMap::new();
    let mut rval = command;

    let mut dcmds = dcmds();

    // add in the repl
    dcmds.push(CommandDescription {
        init: repl::init,
        docmsg: "For additional documentation, run \"humility doc repl\"",
    });

    for dcmd in dcmds {
        let (cmd, subcmd) = (dcmd.init)();

        let name = match cmd {
            Command::Attached { name, .. } => name,
            Command::Unattached { name, .. } => name,
        };

        cmds.insert(name, cmd);

        rval = rval.subcommand(subcmd.after_help(dcmd.docmsg));
    }

    (cmds, rval)
}

pub fn subcommand(
    context: &mut humility::ExecutionContext,
    commands: &HashMap<&'static str, Command>,
) -> Result<()> {
    let Subcommand::Other(subargs) = context.cli.cmd.as_ref().unwrap();
    let subargs = subargs[0].clone();

    let command = commands.get(&*subargs)
        .with_context(|| format!("command {} not found", subargs))?;

    let archive = match command {
        Command::Attached { archive, .. } => archive,
        Command::Unattached { archive, .. } => archive,
    };

    let mut hubris =
        HubrisArchive::new().context("failed to initialize")?;

    if *archive != Archive::Ignored {
        if let Some(archive) = &context.cli.archive {
            hubris.load(archive).with_context(|| {
                format!("failed to load archive \"{}\"", archive)
            })?;
        } else if let Some(dump) = &context.cli.dump {
            hubris.load_dump(dump).with_context(|| {
                format!("failed to load dump \"{}\"", dump)
            })?;
        }
    }

    if *archive == Archive::Required && !hubris.loaded() {
        bail!("must provide a Hubris archive or dump");
    }

    context.archive = Some(hubris);

    match command {
        Command::Attached { run, attach, validate, .. } => {
            humility_cmd::attach(
                context,
                *attach,
                *validate,
                |context| (run)(context),
            )
        }
        Command::Unattached { run, .. } => {
            (run)(context)
        }
    }
}
