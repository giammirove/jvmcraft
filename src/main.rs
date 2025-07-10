mod class_loader;
mod native;
mod runtime;
mod utils;

use clap::Parser;
use class_loader::class_file;
use color_eyre::eyre::{eyre, Result};
use utils::get_env;

use std::sync::{Arc, Mutex};
use tracing_subscriber::reload;
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// JVM to play Minecraft !
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// User Module (folder with the code to execute)
  #[arg(short, long)]
  user_module: String,

  /// Main Class
  #[arg(short, long)]
  class_name: String,

  /// Method Name (i.e. main)
  #[arg(short, long)]
  method_name: String,

  /// Method Descriptor (i.e. ([Ljava/lang/String;)V) [No arguments supported yet]
  #[arg(short, long)]
  descriptor: String,

  #[arg(short, long, default_value_t = false)]
  verbose: bool,
}

fn main() -> Result<()> {
  let args = Args::parse();

  if get_env("JMODS", "").is_empty() {
    return Err(eyre!("[!] JMODS is not set ... aborting"));
  }

  let filter = {
    if args.verbose {
      filter::LevelFilter::DEBUG
    } else {
      filter::LevelFilter::ERROR
    }
  };
  let (reload_filter, reload_handle) = reload::Layer::new(filter);

  let fmt_layer = fmt::layer()
    .without_time()
    .with_file(false)
    .with_target(false)
    .with_line_number(false);

  tracing_subscriber::registry()
    .with(reload_filter)
    .with(fmt_layer)
    .init();

  let handle = Arc::new(Mutex::new(reload_handle));

  let mut jvm = runtime::jvm::JVM::build(&args.user_module, vec![])?;
  jvm.set_logging_handle(handle);

  // example:  jvm.push_frame_from_class("SocketClient", "main", "([Ljava/lang/String;)V",
  // vec![])?;
  jvm.push_frame_from_class(
    &args.class_name,
    &args.method_name,
    &args.descriptor,
    vec![],
  )?;

  jvm.run()?;

  Ok(())
}
