//! This module provides arguments related to the user account config.

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use email::folder::sync::FolderSyncStrategy;
use log::info;
use std::collections::HashSet;

use crate::{folder, ui::table};

const ARG_ACCOUNT: &str = "account";
const ARG_DRY_RUN: &str = "dry-run";
const ARG_RESET: &str = "reset";
const CMD_ACCOUNT: &str = "account";
const CMD_CONFIGURE: &str = "configure";
const CMD_LIST: &str = "list";
const CMD_SYNC: &str = "sync";

type DryRun = bool;
type Reset = bool;

/// Represents the account commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Represents the list accounts command.
    List(table::args::MaxTableWidth),
    /// Represents the sync account command.
    Sync(Option<FolderSyncStrategy>, DryRun),
    /// Configure the current selected account.
    Configure(Reset),
}

/// Represents the account command matcher.
pub fn matches(m: &ArgMatches) -> Result<Option<Cmd>> {
    let cmd = if let Some(m) = m.subcommand_matches(CMD_ACCOUNT) {
        if let Some(m) = m.subcommand_matches(CMD_CONFIGURE) {
            info!("configure account subcommand matched");
            let reset = parse_reset_flag(m);
            Some(Cmd::Configure(reset))
        } else if let Some(m) = m.subcommand_matches(CMD_LIST) {
            info!("list accounts subcommand matched");
            let max_table_width = table::args::parse_max_width(m);
            Some(Cmd::List(max_table_width))
        } else if let Some(m) = m.subcommand_matches(CMD_SYNC) {
            info!("sync account subcommand matched");
            let dry_run = parse_dry_run_arg(m);
            let include = folder::args::parse_include_arg(m);
            let exclude = folder::args::parse_exclude_arg(m);
            let folders_strategy = if let Some(folder) = folder::args::parse_global_source_arg(m) {
                Some(FolderSyncStrategy::Include(HashSet::from_iter([
                    folder.to_owned()
                ])))
            } else if !include.is_empty() {
                Some(FolderSyncStrategy::Include(include.to_owned()))
            } else if !exclude.is_empty() {
                Some(FolderSyncStrategy::Exclude(exclude))
            } else if folder::args::parse_all_arg(m) {
                Some(FolderSyncStrategy::All)
            } else {
                None
            };
            Some(Cmd::Sync(folders_strategy, dry_run))
        } else {
            None
        }
    } else {
        None
    };

    Ok(cmd)
}

/// Represents the account subcommand.
pub fn subcmd() -> Command {
    Command::new(CMD_ACCOUNT)
        .about("Subcommand to manage accounts")
        .long_about("Subcommand to manage accounts like configure, list or sync")
        .aliases(["accounts", "acc"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new(CMD_CONFIGURE)
                .about("Configure the given account")
                .aliases(["config", "conf", "cfg"])
                .arg(reset_flag())
                .arg(folder::args::source_arg(
                    "Define the account to be configured",
                )),
        )
        .subcommand(
            Command::new(CMD_LIST)
                .about("List all accounts")
                .long_about("List all accounts that are set up in the configuration file")
                .arg(table::args::max_width()),
        )
        .subcommand(
            Command::new(CMD_SYNC)
                .about("Synchronize the given account locally")
                .arg(folder::args::all_arg("Synchronize all folders"))
                .arg(folder::args::include_arg(
                    "Synchronize only the given folders",
                ))
                .arg(folder::args::exclude_arg(
                    "Synchronize all folders except the given ones",
                ))
                .arg(dry_run()),
        )
}

/// Represents the user account name argument. This argument allows
/// the user to select a different account than the default one.
pub fn global_args() -> impl IntoIterator<Item = Arg> {
    [Arg::new(ARG_ACCOUNT)
        .help("Override the default account")
        .long_help(
            "Override the default account

The given account will be used by default for all other commands (when applicable).",
        )
        .long("account")
        .short('a')
        .global(true)
        .value_name("name")]
}

/// Represents the user account name argument parser.
pub fn parse_global_arg(matches: &ArgMatches) -> Option<&str> {
    matches.get_one::<String>(ARG_ACCOUNT).map(String::as_str)
}

/// Represents the user account sync dry run flag. This flag allows
/// the user to see the changes of a sync without applying them.
pub fn dry_run() -> Arg {
    Arg::new(ARG_DRY_RUN)
        .help("Do not apply changes of the synchronization")
        .long_help(
            "Do not apply changes of the synchronization.
Changes can be visualized with the RUST_LOG=trace environment variable.",
        )
        .short('d')
        .long("dry-run")
        .action(ArgAction::SetTrue)
}

/// Represents the user account sync dry run flag parser.
pub fn parse_dry_run_arg(m: &ArgMatches) -> bool {
    m.get_flag(ARG_DRY_RUN)
}

pub fn reset_flag() -> Arg {
    Arg::new(ARG_RESET)
        .help("Reset the configuration")
        .short('r')
        .long("reset")
        .action(ArgAction::SetTrue)
}

pub fn parse_reset_flag(m: &ArgMatches) -> bool {
    m.get_flag(ARG_RESET)
}
