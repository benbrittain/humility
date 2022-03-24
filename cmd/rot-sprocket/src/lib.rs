// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use humility::core::Core;
use humility::hubris::*;
use humility_cmd::hiffy::*;
use humility_cmd::{Archive, Args, Attach, Command, Validate};

use anyhow::{anyhow, bail, Result};
use clap::{App, ArgGroup, IntoApp, Parser};
use hif::*;

use sprockets::msgs::{
    deserialize, serialize, RotOpV1, RotRequest, RotResponse, RotResultV1,
    SerializedSize,
};

extern crate log;

#[derive(Parser, Debug)]
#[clap(
    name = "rot-sprocket", about = env!("CARGO_PKG_DESCRIPTION"),
    group = ArgGroup::new("command").multiple(false),
    group = ArgGroup::new("sending").multiple(false)
)]
struct RotSprocketArgs {
    /// Get all endorsed public keys
    #[clap(long, short, group = "command")]
    get_endorsements: bool,

    /// sets timeout
    #[clap(
        long, short = 'T', default_value = "5000", value_name = "timeout_ms",
        parse(try_from_str = parse_int::parse)
     )]
    timeout: u32,
}

fn rot_sprocket(
    hubris: &HubrisArchive,
    core: &mut dyn Core,
    _args: &Args,
    subargs: &[String],
) -> Result<()> {
    let subargs = RotSprocketArgs::try_parse_from(subargs)?;
    let mut context = HiffyContext::new(hubris, core, subargs.timeout)?;
    let funcs = context.functions()?;

    let cmd = if subargs.get_endorsements {
        funcs.get("RotSprocketGetEndorsements", 1)?
    } else {
        bail!("No rot-sprocket command given. Try ... rot-sprocket --help");
    };

    let mut reqbuf = [0u8; RotRequest::MAX_SIZE];
    let req = RotRequest { id: 0, version: 1, op: RotOpV1::GetEndorsements };
    let size = serialize(&mut reqbuf, &req).unwrap();

    println!("RotResponse::MAX_SIZE = {}", RotResponse::MAX_SIZE);

    let mut ops = vec![];
    ops.push(Op::Push32(size as u32));
    ops.push(Op::Call(cmd.id));
    ops.push(Op::Done);

    let mut results =
        context.run(core, ops.as_slice(), Some(&reqbuf[..size]))?;

    let rspbuf = results.pop().unwrap().unwrap();
    let (rsp, _) = deserialize::<RotResponse>(&rspbuf).unwrap();
    println!("Received {:#?}", rsp);

    Ok(())
}

pub fn init() -> (Command, App<'static>) {
    (
        Command::Attached {
            name: "rot-sprocket",
            archive: Archive::Required,
            attach: Attach::LiveOnly,
            validate: Validate::Booted,
            run: rot_sprocket,
        },
        RotSprocketArgs::into_app(),
    )
}
