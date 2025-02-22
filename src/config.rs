use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

use confique::Config;

#[derive(Config)]
pub struct ProgrsConfig {
  /// The WoW Log directory
  pub watchdir: String,
  /// The directory the videos are safed in
  pub viddir: String,
  /// Configuration of the command used for recording. Default uses
  /// gpu-screen-recorder, which I can recommend. Use
  /// gpu-screen-recorder-gtk to find the right configuration options for your
  /// system.
  #[config(nested)]
  pub recorder: RecorderConfig,
  /// The path to mkvmerge. This is used to merge chapter markers into the
  /// video, for now deaths of players are supported. Assumes `mkvmerge` can
  /// handle the output format of the configured recorder for this. If you don't
  /// want/need this, simply put an empty string here.
  #[config(default = "/usr/bin/mkvmerge")]
  pub mkvmerge: String,
}

#[derive(Config)]
pub struct RecorderConfig {
  /// Full path of the binary to call
  #[config(default = "/usr/bin/gpu-screen-recorder",
    validate = executable)]
  pub command: String,
  /// Arguments to use for recording. Array of Strings, which are passed to the
  /// binary in order. Skip the switch that designates the output file, that
  /// goes into `outputswitch`
  #[config(default = [
    "-w", "DisplayPort-0",
    "-c", "mkv",
    "-k", "hevc",
    "-ac", "opus",
    "-f", "60",
    "-cursor", "yes",
    "-restore-portal-session", "yes",
    "-cr", "limited",
    "-encoder", "gpu",
    "-q", "very_high",
    "-a", "device:default_output"
  ])]
  pub args: Vec<String>,
  /// The command line switch to designate the output file. If the output file
  /// is the last argument without a switch, simply put an empty string here.
  #[config(default = "-o")]
  pub outputswitch: String
}

pub fn executable(s: &String) -> Result<(), &'static str> {
  let p: PathBuf = s.into();

  let Ok(metadata) = p.metadata() else {
    return Err("Could not determine metadata");
  };


  if !metadata.is_file() {
    return Err("not a file");
  } 

  let permissions = metadata.permissions();

  if permissions.mode() & 0o111 == 0 {
    return Err("not executable");
  }

  Ok(())
}
