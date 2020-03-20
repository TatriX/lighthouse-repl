use ansi_term::Color;
use lighthouse::{state, HueBridge};
use palette::{Component, Hsl, Srgb, Yxy};
use std::f32::consts::PI;
use std::thread::sleep;
use std::time::Duration;
use std::thread;
use rand::{thread_rng, Rng};
use crossbeam_channel::{unbounded, Sender};

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
        println!("It's test!");
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
                    let hue: Hsl<_, f32> =
                        Hsl::new(step.to_degrees() + shift * index as f32, 0.8, 0.5);

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

pub struct RandomHueLoop;

impl Loop for RandomHueLoop {
    fn name(&self) -> &'static str {
        "random-hue"
    }

    fn play(&self, bridge: &HueBridge) {
        let (s, r) = unbounded();
        let lights: [u8; 3] = [2, 3, 4];

        for (index, light) in lights.iter().enumerate() {
            Self::spawn(s.clone(), *light, index);
        }

        for (light, hsl) in r {
            Self::send_hsl(bridge, light, hsl);
        }
    }
}

impl RandomHueLoop {
    fn spawn(chan: Sender<(u8, Hsl)>, light: u8, index: usize) {
        thread::spawn(move || {
            loop {
                let shift = thread_rng().gen_range(35, 140);

                for step in Steps::new(24) {
                    let hsl = Hsl::new(step.to_degrees() + shift as f32 * index as f32, 0.8, 0.5);

                    let (h, s, l) = hsl.into_components();
                    let rgb: Srgb<u8> = Srgb::from(hsl).into_format();
                    let (r, g, b) = rgb.into_components();

                    let dur = thread_rng().gen_range(4, 16);
                    println!(
                        "# {} | {} | (for {} sec)",
                        light,
                        Color::RGB(r, g, b).paint(format!(
                            "██████ {:3} {:.1} {:.1}",
                            h.to_degrees().round(),
                            s,
                            l
                        )),
                        dur,
                    );

                    chan.send((light, hsl)).unwrap();
                    sleep(Duration::from_millis(dur * 1000));
                }
            }
        });
    }

    fn send_hsl(bridge: &HueBridge, light: u8, hsl: Hsl) {
        let dur = 5;
        let yxy = Yxy::from(hsl);

        bridge
            .state_by_ids(
                &[light],
                state!(
                    on: true,
                    bri: yxy.luma.convert(),
                    xy: [yxy.x.convert(), yxy.y.convert()],
                    transitiontime: dur
                ),
            )
            .unwrap();
    }
}

pub struct Steps {
    num: i8,
    index: i8,
}

impl Steps {
    pub fn new(num: i8) -> Self {
        Self { num, index: 0 }
    }
}

impl Iterator for Steps {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.num {
            let index = self.index as f32;
            self.index += 1;
            Some(index * 2. * PI / f32::from(self.num))
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
