// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ## `humility repl`
//!
//! read, eval, print, loop

use anyhow::Result;
use humility::core::Core;
use humility::hubris::*;
use humility_cmd::{Archive, Args, Attach, Command, Validate};
use clap::Command as ClapCommand;

use std::io::{self, Write};

#[derive(Default)]
struct State {
    history: Vec<String>,
}

fn repl(
    context: &mut humility::Context,
    _hubris: &HubrisArchive,
    _args: &Args,
    _subargs: &[String],
) -> Result<()> {
    let mut state = State::default();
    let mut input = String::new();

    println!("Welcome to the humility REPL! Try out some subcommands, or 'quit' to quit!");
    loop {
        print!("> ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut input)?;
        let result = eval(context, &state, &input)?;
        println!("{}", result);

        state.history.push(input.clone());
        input.clear();
    }
}

fn eval(context: &mut humility::Context, state: &State, input: &str) -> Result<String> {
    match input.trim() {
        "quit" => {
            println!("Quitting!");
            std::process::exit(0);
        }
        "history" => Ok(state.history.join("").trim().to_string()),
        user_input => {
            let mut input = vec!["humility"];
            input.extend(user_input.split(' '));

            let (commands, _, args) = crate::parse_args(input);
            if let Err(e) = crate::execute_subcommand(context, commands, args) {
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
        Command::Attached {
            name: "repl",
            archive: Archive::Required,
            attach: Attach::Any,
            validate: Validate::Match,
            run: repl,
        },
        ClapCommand::new("repl"),
    )
}
