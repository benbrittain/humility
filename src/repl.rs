// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ## `humility repl`
//!
//! read, eval, print, loop

use anyhow::Result;
use clap::Command as ClapCommand;
use humility_cmd::{ArchiveRequired, Attach, Validate, AttachementMetadata, Command};

use std::io::{self, Write};

use crate::cmd;

fn repl(
    context: &mut humility::ExecutionContext,
) -> Result<()> {
    let mut input = String::new();

    println!("Welcome to the humility REPL! Try out some subcommands, or 'quit' to quit!");
    loop {
        print!("> ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;
        let result = eval(context, &input)?;
        println!("{}", result);

        context.history.push(input.clone());
        input.clear();
    }
}

fn eval(context: &mut humility::ExecutionContext, input: &str) -> Result<String> {
    match input.trim() {
        "quit" => {
            println!("Quitting!");
            std::process::exit(0);
        }
        "history" => Ok(context.history.join("").trim().to_string()),
        user_input => {
            let mut input = vec!["humility"];
            input.extend(user_input.split(' '));

            let (commands, _, cli) = crate::parse_args(input);
            context.cli = cli;
            if let Err(e) = cmd::subcommand(context, &commands) {
                Ok(format!(
                    "I'm sorry, Dave. I'm afraid I can't understand that. {e}",
                ))
            } else {
                Ok(String::from("It worked!"))
            }
        }
    }
}

pub fn init() -> (Command, ClapCommand<'static>) {
    (
        Command {
            name: "repl",
            archive: ArchiveRequired::Required,
            attatchment_metadata: Some(AttachementMetadata {
                attach: Attach::Any,
                validate: Validate::Match,
            }),
            run: repl,
        },
        ClapCommand::new("repl"),
    )
}
