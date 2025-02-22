use std::{error::Error, fs::{self, create_dir_all}, io, path::Path};

use config::ProgrsConfig;
use confique::{toml::template, Config, toml::FormatOptions};
use dirwatcher::DirWatcher;
use recorder::Recorder;
use directories::ProjectDirs;

const PREFIX: &[u8] = b"WoWCombatLog-";

pub mod config;
//pub mod follow;
pub mod dirwatcher;
pub mod events;
pub mod parser;
pub mod recorder;

pub async fn main() -> Result<(), io::Error> {
  let Some(dirs) = ProjectDirs::from("", "", "progrs") else {
    return Err(io::Error::other("Could not determine config directory, exiting"));
  };
  let confdir = dirs.config_dir();
  if create_dir_all(&confdir).is_err() {
    return Err(io::Error::other("Could not create config directory, exiting"));
  }

  let conffile = Path::new(&confdir).join("config.toml");

  if !(fs::exists(&conffile)?) {
    println!("Config file {} does not exist, creating with default values. \
    Please adjust to your needs and run progrs again",
    conffile.to_string_lossy());

    let toml = template::<ProgrsConfig>(FormatOptions::default());
    fs::write(&conffile, &toml)?;
    return Ok(());
  };

  let conf = match ProgrsConfig::from_file(&conffile) {
    Ok(c) => c,
    Err(e) => {
      eprintln!("Error: {e}");

      let mut e: &dyn Error = &e;
      while let Some(err) = e.source() {
        eprintln!("Because of: {err}");
        e = err;
      }
    return Err(io::Error::other("Error reading config file"));
    }
  };

  let mut recorder = Recorder::new(
    conf.viddir,
    conf.recorder.command,
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
