use ansi_term::Color;
use lighthouse::{state, HueBridge};
use palette::{Component, Hsl, Srgb, Yxy};
use std::f64::consts::PI;
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

    fn play(&self, _: &HueBridge) {
        todo!()
    }
}

pub struct SoloHueLoop;

impl Loop for SoloHueLoop {
    fn name(&self) -> &'static str {
        "solo-hue"
    }

    fn play(&self, bridge: &HueBridge) {
        loop {
            let dur = 5;
            let shift = 75.;
            let lights: &[u8] = &[2, 3, 4];
            for step in Steps::new(24) {
                for (index, light) in lights.iter().enumerate() {
                    let hue: Hsl<_, f64> =
                        Hsl::new(step.to_degrees() + shift * index as f64, 0.8, 0.5);

                    let (h, s, l) = hue.into_components();
                    let rgb: Srgb<u8> = Srgb::from(hue).into_format();
                    let (r, g, b) = rgb.into_components();

                    println!(
                        "{}",
                        Color::RGB(r, g, b).paint(format!(
                            "██████ {:3} {:.1} {:.1}",
                            h.to_degrees().round(),
                            s,
                            l
                        ))
                    );

                    let yxy = Yxy::from(hue);

                    bridge
                        .state_by_ids(
                            &[*light],
                            state!(
                                on: true,
                                bri: yxy.luma.convert(),
                                xy: [yxy.x.convert(), yxy.y.convert()],
                                transitiontime: dur
                            ),
                        )
                        .unwrap();
                }

                println!();
                sleep(Duration::from_millis(dur as u64 * 1000));
            }
        }
    }
}

pub struct Steps {
    num: i32,
    index: i32,
}

impl Steps {
    pub fn new(num: i32) -> Self {
        Self { num, index: 0 }
    }
}

impl Iterator for Steps {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.num {
            let index = self.index as f64;
            self.index += 1;
            Some(index * 2. * PI / f64::from(self.num))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steps() {
        let expected = (0..360).step_by(15).collect::<Vec<_>>();
        let got = Steps::new(24)
            .map(|step| step.to_degrees().round() as usize)
            .collect::<Vec<_>>();
        assert_eq!(got, expected);
    }
}
