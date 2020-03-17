use lighthouse::{colors::*, state, HueBridge};
use std::thread::sleep;
use std::time::Duration;

/// Loopable sequence of commands
pub trait Loop {
    fn play(&self, h: &HueBridge);
    fn name(&self) -> &'static str;
}

pub struct TestLoop;

impl Loop for TestLoop {
    fn name(&self) -> &'static str {
        "test"
    }

    fn play(&self, h: &HueBridge) {
        loop {
            let steps = 30;
            let dur = 30;
            for i in 0..steps {
                // 0 .. 1
                // 0 .. 9
                let color =
                    palette::Hsl::new(f64::from(i * steps) / 2. * std::f64::consts::PI, 0.5, 0.5);
                let (r, g, b) = palette::Srgb::from(color).into_components();
                println!("{} {} {}", r, g, b);
                let (r, g, b) = ((r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8);
                println!("Setting color to {:?} from {:?}", (r, g, b), color);
                h.state_by_ids(
                    &[2, 3, 4],
                    state!(on: true, bri: 125, xy: rgb_to_xy(r, g, b), transitiontime: dur),
                )
                .unwrap();

                sleep(Duration::from_millis(dur as u64 * 1000));
            }
        }
    }
}
