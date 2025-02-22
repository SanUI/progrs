pub struct Config {
  pub watchdir: &'static str,
  pub viddir: &'static str,
  pub command: &'static str,
  pub mkvmerge: &'static str,
}

impl Config {
  pub fn new(
    viddir: &'static str,
    watchdir: &'static str,
    command: &'static str,
    mkvmerge: &'static str,
  ) -> Self {
    Self {
      viddir,
      watchdir,
      command,
      mkvmerge,
    }
  }
}
