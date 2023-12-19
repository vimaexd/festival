pub fn ascii_bar(val: u32, max: u32) -> String {
  let mut s = String::new();
  for _ in 0..val {
    s.push('█')
  }

  for _ in 0..(max-val) {
    s.push('░')
  }
  return s;
}