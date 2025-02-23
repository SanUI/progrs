use std::{
  ffi::{OsStr, OsString},
  fs::File,
  io::{self, Read, Seek, SeekFrom},
  os::unix::ffi::OsStrExt,
  path::{Path, PathBuf},
};

use futures_util::StreamExt;
use inotify::{EventMask, EventStream, Inotify, WatchDescriptor, WatchMask};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{events::Event, parser::Parser, PREFIX};

pub struct DirWatcher {
  dir: PathBuf,
  dirwatcher: Option<WatchDescriptor>,
  file: PathBuf,
  filewatcher: Option<WatchDescriptor>,
  fd: File,
  sender: Sender<Event>,
}

impl DirWatcher {
  pub fn at(dir: &str) -> io::Result<Receiver<Event>> {
    let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel(1);

    let dir: PathBuf = dir.into();
    let mut file = dir.clone();
    file.push(get_newest_file(&dir)?);
    let fd = open_file(&file, SeekFrom::End(0))?;

    println!("Now watching {}", file.to_string_lossy());

    let dirwatcher = Self {
      dir,
      dirwatcher: None,
      file,
      filewatcher: None,
      fd,
      sender: tx,
    };
    tokio::spawn(dirwatcher.watch());
    Ok(rx)
  }

  async fn watch(mut self) {
    let mut buffer = [0; 1024];
    let mut parsebuf = vec![];

    loop {
      let mut stream = match self.create_inotify(&mut buffer) {
        Ok(s) => s,
        Err(e) => {
          self.sender.send(Event::IoErr(e)).await.unwrap();
          return;
        }
      };

      while let Some(Ok(event)) = stream.next().await {
        match event.mask {
          EventMask::CREATE if Some(&event.wd) == self.dirwatcher.as_ref() => {
            let fname: Option<&OsStr> = event.name.as_deref();

            match fname {
              Some(f) if f.as_bytes().starts_with(PREFIX) => {
                self.file.set_file_name(f);
                self.fd = match open_file(&self.file, SeekFrom::Start(0)) {
                  Ok(f) => f,
                  Err(e) => {
                    self.sender.send(Event::IoErr(e)).await.unwrap();
                    return;
                  }
                };
                println!("Now watching {}", self.file.to_string_lossy());
                self.on_modify(&mut parsebuf).await;
                break;
              }
              _ => {}
            }
          }
          EventMask::MODIFY if Some(&event.wd) == self.filewatcher.as_ref() => {
            //println!("File {} got modified", log.path.display());
            self.on_modify(&mut parsebuf).await;
          }
          _ => {}
        }
      }
    }
  }

  fn create_inotify<'a, 'b: 'a>(
    &'a mut self,
    b: &'b mut [u8],
  ) -> io::Result<EventStream<&'b mut [u8]>> {
    let inotify = Inotify::init().expect("Failed to initialize inotify");

    self.dirwatcher =
      Some(inotify.watches().add(&self.dir, WatchMask::CREATE)?);
    self.filewatcher =
      Some(inotify.watches().add(&self.file, WatchMask::MODIFY)?);

    inotify.into_event_stream(b)
  }

  async fn on_modify(&mut self, parsebuf: &mut Vec<u8>) {
    self.fd.read_to_end(parsebuf).unwrap();
    let mut slice: &[u8] = &*parsebuf;
    let p = Parser::new();
    p.parse(&mut slice, self.sender.clone()).await;

    if !slice.is_empty() {
      let chopoff = parsebuf.len() - slice.len();
      parsebuf.drain(..chopoff);
    } else {
      parsebuf.clear();
    }
    //println!("Remaining buffer: '{}'", String::from_utf8_lossy(parsebuf));
    //println!("----------");
  }
}

fn open_file(file: &Path, at: SeekFrom) -> io::Result<File> {
  let mut fd = File::open(file)?;
  fd.seek(at).expect("File should be seekable");
  Ok(fd)
}

fn get_newest_file(dir: &PathBuf) -> io::Result<OsString> {
  std::fs::read_dir(dir)
    .expect("Couldn't access watch directory")
    .flatten() // Remove failed
    .filter(|f| {
      let name = f.file_name();
      f.metadata().map(|m| {
        m.is_file() && name.as_bytes().starts_with(PREFIX)
      })
      .unwrap_or(false)
    })
    .max_by_key(|x| {
      x.metadata()
       .expect("Metadata was available")
       .modified()
       .expect("Should be available on linux")
    })
    .map(|x| x.file_name())
    .ok_or_else(|| {
      io::Error::other("Could not find newest logfile")
    })
}
