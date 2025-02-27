use std::io;

use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum Event {
  EncounterStart(NaiveDateTime, String),
  EncounterEnd,
  PlayerDeath(NaiveDateTime, String),
  ChallengeModeStart(NaiveDateTime, String),
  ChallengeModeEnd,
  //  NewFile(PathBuf),
  IoErr(io::Error),
}
