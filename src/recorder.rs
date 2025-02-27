use std::{
  fmt::{Display, Write as FmtWrite},
  fs::{self, remove_file},
  process::{Child, Command, Stdio},
};

use chrono::NaiveDateTime;
use nix::{
  sys::signal::{kill, Signal},
  unistd::Pid,
};

use crate::{config::executable, events::Event};

pub struct Recorder {
  pub viddir: String,
  pub command: String,
  pub mkvmerge: Option<String>,
  pub recording: Option<Recording>,
}

pub struct Recording {
  starttime: NaiveDateTime,
  filename: String,
  events: Vec<Event>,
  process: Child,
  pub activity: Activity,
}

pub enum Activity {
  /// Raidboss with name
  Raid(String),
  /// Mythic+ Dungeon with dungeon name
  MythicPlus(String)
}

impl Display for Activity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Self::Raid(s) | Self::MythicPlus(s) => write!(f, "{s}")
      }
    }
}


impl Recorder {
  pub fn new(
    viddir: String,
    command: String,
    mkvmerge: String,
  ) -> Self {
    let mut mkvm = None;

    if executable(&mkvmerge).is_ok() {
      mkvm = Some(mkvmerge);
    }

    Self {
      viddir,
      command,
      mkvmerge: mkvm,
      recording: None,
    }
  }

  pub fn start_recording(&mut self, time: NaiveDateTime, activity: Activity) {
    let datetimestr = time.format("%Y%m%d_%H%M%S");
    let filename = format!("{datetimestr}_{activity}");
    println!("Recording into {filename}");
    let viddir = &self.viddir;


    let recorder = Command::new(&self.command)
      .args(["-w", "DisplayPort-0"])
      .args(["-c", "mkv"])
      .args(["-k", "hevc"])
      .args(["-ac", "opus"])
      .args(["-f", "60"])
      .args(["-cursor", "yes"])
      .args(["-restore-portal-session", "yes"])
      .args(["-cr", "limited"])
      .args(["-encoder", "gpu"])
      .args(["-q", "very_high"])
      .args(["-a", "device:default_output"])
      .args(["-o", &format!("{viddir}/{filename}.mkv")])
      .stderr(Stdio::null())
      .stdin(Stdio::null())
      .spawn()
      .expect("Spawning gpu-screen-recorder");

    let recording = Recording::new(time, filename, recorder, activity);
    self.recording = Some(recording);
  }

  pub fn stop_recording(&mut self) {
    let Some(recording) = self.recording.take() else {
      println!("Not recording, can't stop it");
      return;
    };

    let deaths = recording.create_chapters(&recording.starttime);
    let filename = recording.filename;
    let viddir = self.viddir.clone();
    let mut process = recording.process;
    let mkvmerge = self.mkvmerge.clone();

    tokio::spawn(async move {
      let pid = process.id();
      println!("Killing recorder {pid}");
      kill(
        Pid::from_raw(i32::try_from(pid).expect("Pid conversion to i32")),
        Signal::SIGINT,
      )
      .expect("Killing process failed");

      let exitstatus = process.wait().expect("Waiting for recorder exit");

      if !exitstatus.success() {
        println!("Recorder exited with status {exitstatus}");
        return;
      }

      if let Some(mergecommand) = mkvmerge {
        let chapterfile = format!("{viddir}/{filename}.txt");
        fs::write(&chapterfile, deaths).expect("Writing chapter file");

        let mkvfile = format!("{viddir}/{filename}.mkv");
        let outfile = format!("{viddir}/{filename}_final.mkv");

        let mergestatus = Command::new(mergecommand)
          .args(["--chapters", &chapterfile])
          .args(["-o", &outfile])
          .args([&mkvfile])
          .stdout(Stdio::null())
          .status()
          .expect("Merging failed");

        if mergestatus.success() {
          remove_file(&mkvfile).expect("File was created");
          remove_file(&chapterfile).expect("File was created");
        } else {
          println!("Merge exited with status {mergestatus}, keeping \
                    intermediate files");
        }
      }
    });
  }
}

impl Recording {
  pub fn new(
    starttime: NaiveDateTime,
    filename: String,
    process: Child,
    activity: Activity
  ) -> Self {
    Self {
      starttime,
      filename,
      events: vec![],
      process,
      activity
    }
  }

  pub fn is_raid(&self) -> bool {
    match self.activity {
      Activity::Raid(_) => true,
      Activity::MythicPlus(_) => false
    }
  }

  pub fn is_mythicplus(&self) -> bool {
    match self.activity {
      Activity::Raid(_) => false,
      Activity::MythicPlus(_) => true
    }
  }

  pub fn add_death(&mut self, datetime: NaiveDateTime, name: String) {
    self.events.push(Event::PlayerDeath(datetime, name));
  }

  pub fn add_encounter(&mut self, datetime: NaiveDateTime, name: String) {
    self.events.push(Event::EncounterStart(datetime, name));
  }

  pub fn create_chapters(&self, starttime: &NaiveDateTime) -> String {
    let mut s = String::new();

    for (idx, event) in self.events.iter().enumerate() {
      match event {
        Event::PlayerDeath(time, name) => {
          let tdelta = *time - *starttime;
          writeln!(
            &mut s,
            "CHAPTER{:02}={:02}:{:02}:{:02}.{:03}",
            idx + 1,
            tdelta.num_hours(),
            tdelta.num_minutes() % 60,
            tdelta.num_seconds() % 60,
            tdelta.num_milliseconds() % 1000
          )
          .expect("Write into String");
          writeln!(&mut s, "CHAPTER{:02}NAME=Death: {name}", idx + 1)
            .expect("Write into String");
        }
        Event::EncounterStart(time, name) => {
          let tdelta = *time - *starttime;
          writeln!(
            &mut s,
            "CHAPTER{:02}={:02}:{:02}:{:02}.{:03}",
            idx + 1,
            tdelta.num_hours(),
            tdelta.num_minutes() % 60,
            tdelta.num_seconds() % 60,
            tdelta.num_milliseconds() % 1000
          )
          .expect("Write into String");
          writeln!(&mut s, "CHAPTER{:02}NAME=Encounter Start: {name}", idx + 1)
            .expect("Write into String");
        }
        // We don't push anything else anyways
        _ => {}
      }
    }

    s
  }
}
