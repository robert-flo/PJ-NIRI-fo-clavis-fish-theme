use crate::config::ColorPair;
use crate::segment::{capsule, Segment};

pub fn render(colors: ColorPair) -> Segment {
    let current = jiff::Zoned::now();
    capsule(colors, &format!(" {}", current.strftime("%H:%M:%S")))
}
