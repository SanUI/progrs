pub struct Config {
  pub watchdir: String,
  pub viddir: String,
  pub command: String,
  pub mkvmerge: Option<String>,
}

impl Config {
  pub fn new(
    viddir: String,
    watchdir: String,
    command: String,
    mkvmerge: Option<String>,
  ) -> Self {
    Self {
      viddir,
      watchdir,
      command,
      mkvmerge,
    }
  }
}
