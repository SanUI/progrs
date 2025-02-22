use std::io;

use dirwatcher::DirWatcher;
use recorder::Recorder;

const WATCHDIR: &str = "/home/pips/tmp"; //"/home/pips/Games/WoW/World of Warcraft/_retail_/Logs";
const PREFIX: &[u8] = b"W"; //b"WoWCombatLog-";
const VIDDIR: &str = "/home/pips/tmp";
const COMMAND: &str = "/usr/bin/gpu-screen-recorder";
const MKVMERGE: &str = "/usr/bin/mkvmerge";

pub mod config;
//pub mod follow;
pub mod dirwatcher;
pub mod events;
pub mod parser;
pub mod recorder;

pub async fn main() -> Result<(), io::Error> {
  let conf = config::Config::new(WATCHDIR.into(), VIDDIR.into(), COMMAND.into(), Some(MKVMERGE.into()));

  let mut recorder = Recorder::new(
    conf.viddir,
    conf.command,
    conf.mkvmerge,
  );
  let mut dirwatcher = DirWatcher::at(&conf.watchdir)?;

  while let Some(e) = dirwatcher.recv().await {
    println!("Event: '{e:?}'");

    {
      use events::Event::*;
      match e {
        EncounterStart(datetime, name) => {
          recorder.start_recording(datetime, name);
        }
        EncounterEnd => recorder.stop_recording(),
        PlayerDeath(datetime, name) => {
          if let Some(r) = recorder.recording.as_mut() {
            r.add_death(datetime, name);
          }
        }
        IoErr(error) => {
          eprintln!("Error: '{}'", error);
          break;
        }
      }
    }
  }

  println!("Exiting");

  Ok(())
}
