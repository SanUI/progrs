use std::io;

use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum Event {
  EncounterStart(NaiveDateTime, String),
  EncounterEnd,
  PlayerDeath(NaiveDateTime, String),
  //  NewFile(PathBuf),
  IoErr(io::Error),
}
