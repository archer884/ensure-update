use std::{
    collections::HashMap,
    io,
    path::Path,
    process::{self, Command, Stdio},
};

use abseil::Provider;
use clap::Parser;
use jiff::{SignedDuration, Timestamp};

#[derive(Debug, Parser)]
struct Opts {
    // a git repository to update
    repository: String,

    // how long ago can the last update be before we trigger another
    #[arg(default_value_t = 8)]
    max_age: i32,

    // ignore last update time
    #[arg(short, long)]
    force: bool,

    // print command output
    #[arg(long)]
    verbose: bool,
}

fn main() {
    if let Err(e) = run(Opts::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(opts: Opts) -> io::Result<()> {
    // First thing first, we need to check the last runtime of the command for
    // this repository. If the last runtime was within the last n hours, we do
    // NOT need to run. Step one of this process is to grab our runtime table.
    // This table (a hashmap) stores a list of repository names and the last
    // time each one was updated by us.

    let repository_name = get_name(&opts.repository)?;
    let provider = Provider::builder("ensure-update")
        .with_organization("hack-commons")
        .build();

    // We need this to be mutable because we'll be updating it later.
    let mut table: HashMap<String, Timestamp> = provider.load()?.into_inner();

    // If the timestamp associated with our intended repository is newer than
    // opts.max_age, we'll just return. Otherwise, if the timestamp in question
    // is older than opts.max_age OR if there is no such timestamp, we'll
    // continue with the update operation AND AFTER add/update a timestamp for
    // this repository.
    if is_recent(&table, &repository_name, opts.max_age) && !opts.force {
        return Ok(());
    }

    // Before beginning the update process, we'll set our current directory
    // to that of the target repository.
    std::env::set_current_dir(&opts.repository)?;

    // Next up, we'll prepare our git command.
    let mut update_command = build_update_command();

    // In the event that we've passed the --verbose flag, we'll want to print
    // the output of our command to stdout. Otherwise, we'll do this silently.
    // Either way, we won't redirect stderr -- just in case.
    if !opts.verbose {
        update_command.stdout(Stdio::null());
    }

    let result = update_command.status()?;
    if !result.success() {
        return Err(io::Error::other("repository failed to update"));
    }

    // Lastly, we're going to need to update our table with the timestamp of
    // the update we just performed, and we'll store that table using the
    // provider we already created.

    table.insert(repository_name, Timestamp::now());
    provider.store(table)?;

    Ok(())
}

fn is_recent(table: &HashMap<String, Timestamp>, repository_name: &str, max_age: i32) -> bool {
    let Some(&timestamp) = table.get(repository_name) else {
        return false;
    };

    // No idea why Spans can't be compared, but SignedDurations can, so I guess
    // that's what we're gonna use. /shrug
    let elapsed = timestamp.duration_until(Timestamp::now()).abs();
    SignedDuration::from_hours(max_age as i64) > elapsed
}

fn build_update_command() -> Command {
    let mut command = Command::new("git");
    command.arg("pull");
    command
}

fn get_name(repository: &str) -> io::Result<String> {
    let path = Path::new(repository);

    // This is only legal for directories; a repository MUST
    // be a folder.
    if !path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "not a directory"));
    }

    // The only error case here is if the path provided looks like "/"
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "did you attempt to update root?")
    })?;

    // In theory, I'm not best pleased with this solution, because it means
    // that unequal paths could be treated equally if they map to the same
    // lossy string. However, I just don't care enough. None of my repositories
    // have non-ASCII names anyway.
    Ok(name.to_string_lossy().into())
}
