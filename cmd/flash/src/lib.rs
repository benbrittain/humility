// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ## `humility flash`
//!
//! Flashes the target with the image that is contained within the specified
//! archive (or dump).  This merely executes the underlying flashing
//! mechanism (either pyOCD or OpenOCD, depending on the target); if the
//! requisite software is not installed (or isn't in the path), this will
//! fail.  Temporary files are created as part of this process; if they are to
//! be retained, the `-R` (`--retain-temporaries`) flag should be set.
//! To see what would be executed without actually executing any commands,
//! use the `-n` (`--dry-run`) flag.  As a precautionary measure, if
//! the specified archive already appears to be on the target, `humility
//! flash` will fail unless the `-F` (`--force`) flag is set.
//!

use anyhow::{bail, Result};
use clap::Command as ClapCommand;
use clap::{CommandFactory, Parser};
use humility::hubris::*;
use humility_cmd::{Archive, Args, Command};

use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(name = "flash", about = env!("CARGO_PKG_DESCRIPTION"))]
struct FlashArgs {
    /// force re-flashing if archive matches
    #[clap(long, short = 'F')]
    force: bool,

    /// do not actually flash, but show commands and retain any temporary files
    #[clap(long = "dry-run", short = 'n')]
    dryrun: bool,

    /// retain any temporary files
    #[clap(long = "retain-temporaries", short = 'R')]
    retain: bool,
}

//
// This is the Hubris definition
//
#[derive(Debug, Deserialize)]
enum FlashProgram {
    PyOcd(Vec<FlashArgument>),
    OpenOcd(FlashProgramConfig),
}

#[derive(Debug, Deserialize)]
enum FlashProgramConfig {
    Path(Vec<String>),
    Payload(String),
}

#[derive(Debug, Deserialize)]
enum FlashArgument {
    Direct(String),
    Payload,
    FormattedPayload(String, String),
    Config,
}

#[derive(Debug, Deserialize)]
struct FlashConfig {
    program: FlashProgram,
    args: Vec<FlashArgument>,
}

fn flashcmd(
    hubris: &mut HubrisArchive,
    args: &Args,
    subargs: &[String],
) -> Result<()> {
    let flash_config = hubris.load_flash_config()?;
    let subargs = FlashArgs::try_parse_from(subargs)?;

    let config: FlashConfig = ron::from_str(&flash_config.metadata)?;

    // This is incredibly ugly! It also gives us backwards compatibility!

    let chip = match config.program {
        FlashProgram::PyOcd(args) => {
            let s69 = regex::Regex::new(r"lpc55s69").unwrap();
            let s28 = regex::Regex::new(r"lpc55s28").unwrap();
            let mut c: Option<String> = None;
            for arg in args {
                c = match arg {
                    FlashArgument::Direct(s) => {
                        if s69.is_match(&s) {
                            Some("LPC55S69JBD100".to_string())
                        } else if s28.is_match(&s) {
                            Some("LPC55S28JBD64".to_string())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if !c.is_none() {
                    break;
                }
            }

            if c.is_none() {
                bail!("Failed to find chip from pyOCD config");
            }

            c.unwrap()
        }
        FlashProgram::OpenOcd(a) => match a {
            FlashProgramConfig::Payload(d) => {
                let h7 = regex::Regex::new(r"find target/stm32h7").unwrap();
                let f3 = regex::Regex::new(r"find target/stm32f3").unwrap();
                let f4 = regex::Regex::new(r"find target/stm32f4").unwrap();
                let g0 = regex::Regex::new(r"find target/stm32g0").unwrap();

                let mut c: Option<String> = None;

                for s in d.split("\n") {
                    if h7.is_match(&s) {
                        c = Some("STM32H753ZITx".to_string());
                        break;
                    }
                    if f3.is_match(&s) {
                        c = Some("STM32F301C6Tx".to_string());
                        break;
                    }
                    if f4.is_match(&s) {
                        c = Some("STM32F401CBUx".to_string());
                        break;
                    }
                    if g0.is_match(&s) {
                        c = Some("STM32G030C6Tx".to_string());
                        break;
                    }
                }

                if c.is_none() {
                    bail!("Failed to get chip from OpenOCD config");
                }

                c.unwrap()
            }
            _ => bail!("Unexpected config?"),
        },
    };

    //
    // We need to attach to (1) confirm that we're plugged into something
    // and (2) extract serial information.
    //
    let probe = match &args.probe {
        Some(p) => p,
        None => "auto",
    };

    humility::msg!("Attaching to attach to chip {:x?}", chip);
    let mut c = humility::core::attach(probe, &chip)?;
    let core = c.as_mut();

    //
    // We want to actually try validating to determine if this archive
    // already matches; if it does, this command may well be in error,
    // and we want to force the user to force their intent.
    //
    if hubris.validate(core, HubrisValidate::ArchiveMatch).is_ok() {
        if subargs.force {
            humility::msg!(
                "archive appears to be already flashed; forcing re-flash"
            );
        } else {
            bail!(
                "archive appears to be already flashed on attached device; \
                    use -F (\"--force\") to force re-flash"
            );
        }
    }

    let ihex = tempfile::NamedTempFile::new()?;
    std::fs::write(&ihex, flash_config.ihex)?;
    let ihex_path = ihex.path();

    core.load(ihex_path)?;

    humility::msg!("Flash done.");
    Ok(())
}

pub fn init() -> (Command, ClapCommand<'static>) {
    (
        Command::Unattached {
            name: "flash",
            archive: Archive::Required,
            run: flashcmd,
        },
        FlashArgs::command(),
    )
}
