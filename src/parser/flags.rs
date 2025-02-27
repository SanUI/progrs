pub enum Flags {
  ControlPlayer = 0x00000100,
  TypePlayer = 0x00000400,
}

pub trait HasFlag {
  fn has_flag(&self, flag: Flags) -> bool;
}

impl HasFlag for i32 {
  fn has_flag(&self, flag: Flags) -> bool {
    *self & (flag as i32) != 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn various() {
    let f = 0x514;
    let f2 = 0x100;
    let f3 = 0x400;

    assert!(f.has_flag(Flags::ControlPlayer));
    assert!(f.has_flag(Flags::TypePlayer));
    assert!(f2.has_flag(Flags::ControlPlayer));
    assert!(!f2.has_flag(Flags::TypePlayer));
    assert!(!f3.has_flag(Flags::ControlPlayer));
    assert!(f3.has_flag(Flags::TypePlayer));
  }
}

#[test]
fn with_parse() {
  let f = "0x514";
  let f = f.strip_prefix("0x").unwrap();
  let f = i32::from_str_radix(f, 16).unwrap();

  assert!(f == 1300);

  assert!(f.has_flag(Flags::ControlPlayer));
  assert!(f.has_flag(Flags::TypePlayer));
}
