mod class_loader;
mod runtime;
mod utils;
use clap::Parser;
use class_loader::class_file;
use color_eyre::eyre::Result;

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
    let loglevel = {
        if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Off
        }
    };

    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(loglevel)
        .chain(std::io::stdout())
        .apply()?;

    let mut jvm = runtime::jvm::JVM::build(&args.user_module, vec![])?;
    // example:  jvm.push_frame_from_class("SocketClient", "main", "([Ljava/lang/String;)V", vec![])?;
    jvm.push_frame_from_class(
        &args.class_name,
        &args.method_name,
        &args.descriptor,
        vec![],
    )?;
    jvm.run()?;

    Ok(())
}
