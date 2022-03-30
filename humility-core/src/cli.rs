// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{AppSettings, Parser};

#[derive(Parser, Debug)]
#[clap(name = "humility", max_term_width = 80)]
#[clap(global_setting(AppSettings::NoAutoVersion))]
pub struct Cli {
    /// verbose messages
    #[clap(long, short)]
    pub verbose: bool,

    /// print version information
    #[clap(long, short = 'V')]
    pub version: bool,

    /// specific chip on attached device
    #[clap(
        long,
        short,
        env = "HUMILITY_CHIP",
        default_value = "STM32F407VGTx"
    )]
    pub chip: String,

    /// chip probe to use
    #[clap(long, short, env = "HUMILITY_PROBE", conflicts_with = "dump")]
    pub probe: Option<String>,

    /// Hubris archive
    #[clap(long, short, env = "HUMILITY_ARCHIVE")]
    pub archive: Option<String>,

    /// Hubris dump
    #[clap(long, short, env = "HUMILITY_DUMP")]
    pub dump: Option<String>,

    #[clap(subcommand)]
    pub cmd: Option<Subcommand>,
}

#[derive(Parser, Debug)]
pub enum Subcommand {
    #[clap(external_subcommand)]
    Other(Vec<String>),
}
