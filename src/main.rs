use std::path::PathBuf;

use clap::Parser;

mod flutter_application;
use flutter_application::FlutterApplication;

mod flutter_bindings;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The Flutter application code needs to be snapshotted using
    /// the Flutter tools and the assets packaged in the appropriate
    /// location. This can be done for any Flutter application by
    /// running `flutter build bundle` while in the directory of a
    /// valid Flutter project. This should package all the code and
    /// assets in the "build/flutter_assets" directory. Specify this
    /// directory as the first argument to this utility.
    pub asset_bundle_path: PathBuf,
    /// Typically empty. These extra flags are passed directly to the
    /// Flutter engine. To see all supported flags, run
    /// `flutter_tester --help` using the test binary included in the
    /// Flutter tools.
    pub flutter_flags: Vec<String>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let _application = FlutterApplication::new(&args.asset_bundle_path, args.flutter_flags);
}
