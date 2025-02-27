use std::str;

use chrono::NaiveDateTime;
use memchr::{memchr, memmem, memrchr};
use tokio::sync::mpsc::Sender;

use crate::events::Event;

mod flags;

use flags::{Flags, HasFlag};

#[derive(Default)]
pub struct Parser {}

impl Parser {
  pub fn new() -> Self {
    Parser {}
  }

  pub async fn parse(&self, buffer: &mut &[u8], tx: Sender<Event>) {
    let startit = memmem::find_iter(buffer, "ENCOUNTER_START");

    if let Some(startidx) = startit.last() {
      // Have the buffer start at the start of this line
      // If '\n' isn't found, this is the first line, so just keep the buffer
      if let Some(nidx) = memrchr(b'\n', &buffer[..startidx]) {
        *buffer = &buffer[nidx + 1..];
      }

      if self.skip_encounter_end(buffer) {
        return;
      }

      // This is the important part: ENCOUNTER_START, but not ENCOUNTER_END
      let dt = datetime_from_line(buffer);
      let encounter = encounter_from_line(buffer);

      tx.send(Event::EncounterStart(dt, encounter))
        .await
        .expect("Event channel");

      self.skip_to_next_line(buffer);
    }

    if let Some(endidx) = memmem::find_iter(buffer, "ENCOUNTER_END").next() {
      self.handle_deaths(&buffer[..endidx], &tx).await;
      tx.send(Event::EncounterEnd).await.expect("Event channel");
      *buffer = &buffer[endidx..];
      self.skip_to_next_line(buffer);
    } else {
      // No ENCOUNTER_END -> just handle deaths
      self.handle_deaths(buffer, &tx).await;
    }

    //preserve the last line, if it's not complete
    if let Some(nidx) = memrchr(b'\n', buffer) {
      if nidx < buffer.len() {
        *buffer = &buffer[nidx + 1..];
      } else {
        *buffer = &[]
      }
    }
  }

  async fn handle_deaths(&self, buffer: &[u8], tx: &Sender<Event>) {
    let it = memmem::find_iter(buffer, "UNIT_DIED");

    for idx in it {
      let nidx = memrchr(b'\n', &buffer[..idx]);
      let linestart = nidx.map(|i| i + 1).unwrap_or(0);
      let endidx = memchr(b'\n', &buffer[idx..]);
      let lineend = endidx.map(|i| i + idx).unwrap_or(buffer.len());
      let mut line = &buffer[linestart..lineend];
      let datetime = datetime_from_line(&mut line);

      if let Some(idx) = memmem::find_iter(line, ",Player-").next() {
        let line = &line[idx..];
        let nstart = memchr::memchr(b'"', line).expect("Player name");
        let nend =
          memchr::memchr(b'"', &line[nstart + 1..]).expect("Player name");

        let flagstart = nstart + nend + 3;
        let flagend = memchr::memchr(b',', &line[flagstart..]).expect(
          "Player
        flags",
        );
        let flag = str::from_utf8(&line[flagstart..flagstart + flagend])
          .expect("Player flag");
        let flag = flag.strip_prefix("0x").expect("Hexadecimal");
        let flag = i32::from_str_radix(flag, 16).expect("Player flags");

        // UnitUnconsciousAtDeath
        // -2 because of windows' line endings, -1 in case we're at the end of
        // the buffer
        // No false positives because of short-circuiting
        let line_ends_with_0 =
          buffer[lineend - 1] == b'0' || buffer[lineend - 2] == b'0';

        if flag.has_flag(Flags::ControlPlayer)
          && flag.has_flag(Flags::TypePlayer)
          && line_ends_with_0
        {
          let name =
            String::from_utf8_lossy(&line[nstart + 1..nstart + nend + 1]);
          println!("Got death: {name}");
          tx.send(Event::PlayerDeath(datetime, name.to_string()))
            .await
            .expect("Event channel");
        }
      }
    }
  }

  fn skip_encounter_end(&self, buffer: &mut &[u8]) -> bool {
    let endit = memmem::find_iter(buffer, "ENCOUNTER_END");

    if let Some(endidx) = endit.last() {
      *buffer = &buffer[endidx..];
      self.skip_to_next_line(buffer);
      true
    } else {
      false
    }
  }

  fn skip_to_next_line(&self, buffer: &mut &[u8]) {
    if buffer.is_empty() {
      return;
    }

    if let Some(nidx) = memchr(b'\n', buffer) {
      // \n might be the last char in the buffer
      if nidx == buffer.len() - 1 {
        *buffer = &[]
      } else {
        *buffer = &buffer[nidx + 1..];
      }
    } else {
      *buffer = &[];
    }
  }
}

fn encounter_from_line(line: &mut &[u8]) -> String {
  let firstmark = memchr(b'"', line).expect("ENCOUNTER_START name format");
  let secondmark =
    memchr(b'"', &line[firstmark + 1..]).expect("ENCOUNTER_START name format");
  let v: Vec<u8> = line[firstmark + 1..firstmark + secondmark + 1]
    .iter()
    .map(|c| if c == &b' ' { b'_' } else { *c })
    .collect();

  String::from_utf8(v).expect("Valid UTF8")
}

fn datetime_from_line(line: &mut &[u8]) -> NaiveDateTime {
  let firstblank = memchr(b' ', line).expect("Log Time Format");
  let secondblank =
    memchr(b' ', &line[firstblank + 1..]).expect("Log Time Format");
  let dtstr =
    str::from_utf8(&line[..firstblank + secondblank + 1]).expect("Valid UTF-8");
  let datetime = NaiveDateTime::parse_from_str(dtstr, "%-m/%-d/%Y %H:%M:%S%.f")
    .expect("Time format");
  *line = &line[firstblank + secondblank + 1..];

  datetime
}
