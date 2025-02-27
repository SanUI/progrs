use std::{
  error::Error,
  fs::{self, create_dir_all},
  io,
  path::Path,
};

use config::ProgrsConfig;
use confique::{toml::template, toml::FormatOptions, Config};
use ctrlc;
use directories::ProjectDirs;
use dirwatcher::DirWatcher;
use recorder::{Activity, Recorder};

const PREFIX: &[u8] = b"WoWCombatLog-";

pub mod config;
//pub mod follow;
pub mod dirwatcher;
pub mod events;
pub mod parser;
pub mod recorder;

pub async fn main() -> Result<(), io::Error> {
  let Some(dirs) = ProjectDirs::from("", "", "progrs") else {
    return Err(io::Error::other(
      "Could not determine config directory, exiting",
    ));
  };
  let confdir = dirs.config_dir();
  if create_dir_all(&confdir).is_err() {
    return Err(io::Error::other(
      "Could not create config directory, exiting",
    ));
  }

  let conffile = Path::new(&confdir).join("config.toml");

  if !(fs::exists(&conffile)?) {
    println!(
      "Config file {} does not exist, creating with default values. \
    Please adjust to your needs and run progrs again",
      conffile.to_string_lossy()
    );

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

  let mut recorder =
    Recorder::new(conf.viddir, conf.recorder.command, conf.mkvmerge);
  let (mut dirwatcher, tx) = DirWatcher::at(&conf.watchdir)?;

  ctrlc::set_handler(move || {
    tx.blocking_send(events::Event::CtrlC)
      .expect("Ctrl-C channel");
  })
  .expect("Ctrl-C handler");

  while let Some(e) = dirwatcher.recv().await {
    println!("Event: '{e:?}'");

    {
      use events::Event::*;
      match e {
        EncounterStart(datetime, name) => {
          let Some(recording) = recorder.recording.as_mut() else {
            recorder.start_recording(datetime, Activity::Raid(name));
            continue;
          };

          if recording.is_mythicplus() {
            recording.add_encounter(datetime, name);
          } else {
            println!(
              "Got ENCOUNTER_START with name '{name}', but \
               non-mythicplus activity '{}' is still being recorded",
              recorder
                .recording
                .as_ref()
                .expect("Checked for none")
                .activity
            );
          }
        }
        EncounterEnd => {
          if Some(true) == recorder.recording.as_ref().map(|r| r.is_raid()) {
            recorder.stop_recording();
          }
        }
        ChallengeModeStart(datetime, name) => {
          if recorder.recording.is_none() {
            recorder.start_recording(datetime, Activity::MythicPlus(name));
            continue;
          } else {
            println!(
              "Got CHALLENGE_MODE_START with name '{name}', but \
               activity {} is still being recorded!",
              recorder
                .recording
                .as_ref()
                .expect("Checked for none")
                .activity
            );
          }
        }
        ChallengeModeEnd => {
          if Some(true)
            == recorder.recording.as_ref().map(|r| r.is_mythicplus())
          {
            recorder.stop_recording();
          } else {
            println!(
              "Got CHALLENGE_MODE_END, but no mythicplus recording \
                      running"
            );
          }
        }
        PlayerDeath(datetime, name) => {
          if let Some(r) = recorder.recording.as_mut() {
            r.add_death(datetime, name);
          }
        }
        IoErr(error) => {
          eprintln!("Error: '{}'", error);
          break;
        }
        CtrlC => {
          if recorder.recording.is_none() {
            println!("Caught Ctrl-C with no recording running. Exiting");
            break;
          }

          println!("Caught Ctrl-C, stopping current recording");
          recorder.stop_recording();
        }
      }
    }
  }

  println!("Exiting");

  Ok(())
}
