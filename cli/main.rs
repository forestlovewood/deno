// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

use deno::standalone;
use deno::tokio_util;
use deno::colors;
use deno::flags;
use deno::logger;
use deno::unix_util;
use deno::unwrap_or_exit;
use deno::get_subcommand;
use deno::init_v8_flags;
use deno_core::error::AnyError;
use std::io::Write;
use std::env;

pub fn main() {
  #[cfg(windows)]
  colors::enable_ansi(); // For Windows 10
  unix_util::raise_fd_limit();

  let args: Vec<String> = env::args().collect();
  let standalone_res = match standalone::extract_standalone(args.clone()) {
    Ok(Some((metadata, bundle))) => {
      tokio_util::run_basic(standalone::run(bundle, metadata))
    }
    Ok(None) => Ok(()),
    Err(err) => Err(err),
  };
  if let Err(err) = standalone_res {
    eprintln!("{}: {}", colors::red_bold("error"), err.to_string());
    std::process::exit(1);
  }

  let flags = match flags::flags_from_vec(args) {
    Ok(flags) => flags,
    Err(err @ clap::Error { .. })
      if err.kind == clap::ErrorKind::HelpDisplayed
        || err.kind == clap::ErrorKind::VersionDisplayed =>
    {
      err.write_to(&mut std::io::stdout()).unwrap();
      std::io::stdout().write_all(b"\n").unwrap();
      std::process::exit(0);
    }
    Err(err) => unwrap_or_exit(Err(AnyError::from(err))),
  };
  if !flags.v8_flags.is_empty() {
    init_v8_flags(&*flags.v8_flags);
  }

  logger::init(flags.log_level);

  unwrap_or_exit(tokio_util::run_basic(get_subcommand(flags)));
}
