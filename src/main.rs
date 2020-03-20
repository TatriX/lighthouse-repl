use lighthouse::{colors::*, state, HueBridge};
use prettytable::{cell, row, Table};

use rustyline::error::ReadlineError;
use rustyline::Editor;

use ansi_term::Color::*;

mod loops;

use loops::*;

struct Repl {
    bridge: HueBridge,
    loops: Vec<Box<dyn Loop>>,
}

impl Repl {
    fn new() -> Self {
        Self {
            loops: vec![],
            bridge: HueBridge::connect(),
        }
    }

    fn add_loop(mut self, l: impl Loop + 'static) -> Self {
        self.loops.push(Box::new(l));
        self
    }

    fn run(mut self) {
        let mut rl = Editor::<()>::new();

        let history_path = dirs::data_dir()
            .unwrap_or_else(|| "./".into())
            .with_file_name(".lhr_history");

        // Ignore missing history
        let _ = rl.load_history(&history_path);

        loop {
            let readline = rl.readline(">> ");

            match readline {
                Ok(line) => {
                    if let Err(err) = self.process_line(&line) {
                        println!("{}", Red.paint(err));
                    }
                    rl.add_history_entry(line.as_str());
                    {
                        use lighthouse::*;
                        let lights: std::collections::BTreeMap<u8, Light> = self
                            .bridge
                            .request("lights", RequestType::Get, None)
                            .unwrap()
                            .json()
                            .unwrap();
                        let ids: Vec<u8> = lights.keys().cloned().collect();
                        let count = ids.len() as u8;

                        self.bridge.lights = LightCollection { lights, ids, count };
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted");
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        // TODO: move to Drop
        rl.save_history(&history_path).unwrap();
    }

    fn process_line(&self, line: &str) -> Result<(), String> {
        let words: Vec<_> = line.split_whitespace().collect();
        Ok(match &words[..] {
            ["ls"] => self.list_lights(),
            ["all", "on"] => self.all_lights_set_on(true),
            ["all", "off"] => self.all_lights_set_on(false),
            ["on", id] => self.light_set_on(parse_id(id)?, true),
            ["off", id] => self.light_set_on(parse_id(id)?, false),
            ["bri", id, bri] => self.light_set_bri(parse_id(id)?, parse_bri(bri)?),
            ["rgb", id, r, g, b] => self.light_set_color(parse_id(id)?, parse_rgb(r, g, b)?),
            ["play"] => self.play_loop(RandomHueLoop.name()),
            ["play", name] => self.play_loop(&name),
            ["ls", "loops"] => self.list_loops(),
            _ => Err(Yellow
                .paint(format!("Unknown command: {}", Red.paint(line)))
                .to_string())?,
        })
    }

    fn list_lights(&self) {
        let mut table = Table::new();
        table.add_row(row!["id", "on", "name", "bri", "sat", "hue", "xy"]);
        for (id, light) in &self.bridge.lights.lights {
            let on = if light.state.on { "✓" } else { " " };
            table.add_row(row![
                id,
                on,
                light.name,
                light.state.bri,
                light.state.sat,
                light.state.hue,
                format!("{:?}", light.state.xy),
            ]);
        }
        table.printstd();
    }

    fn light_set_on(&self, id: u8, on: bool) {
        if on {
            let bri = self.bridge.lights.lights[&id].state.bri;
            log::debug!("[light_set_on]: setting bri of {} to {}", id, bri);
            self.bridge
                .state_by_ids(&[id], state!(on: on, bri: bri))
                .unwrap();
        } else {
            self.bridge.state_by_ids(&[id], state!(on: false)).unwrap();
        }
    }

    fn all_lights_set_on(&self, on: bool) {
        self.bridge.all(state!(on: on)).unwrap();
    }

    fn light_set_bri(&self, id: u8, bri: u8) {
        log::debug!("[light_set_bri]: setting bri of {} to {}", id, bri);
        self.bridge.state_by_ids(&[id], state!(bri: bri)).unwrap();
    }

    fn light_set_color(&self, id: u8, (r, g, b): (u8, u8, u8)) {
        self.bridge
            .state_by_ids(&[id], state!(xy: rgb_to_xy(r, g, b)))
            .unwrap();
    }

    // Loops

    fn play_loop(&self, name: &str) {
        if let Some(l) = self.loops.iter().find(|l| l.name() == name) {
            l.play(&self.bridge);
        } else {
            println!(
                "{}",
                Yellow.paint(format!("Loop {} not found", Red.paint(name)))
            );
        }
    }

    fn list_loops(&self) {
        for l in &self.loops {
            println!("- {}", l.name());
        }
    }
}

fn main() {
    env_logger::init();
    Repl::new().add_loop(TestLoop).add_loop(SoloHueLoop).add_loop(RandomHueLoop).run();
}

fn parse_id(s: &str) -> Result<u8, String> {
    s.parse()
        .map_err(|err| format!("cannot parse id: {:?}", err))
}

fn parse_bri(s: &str) -> Result<u8, String> {
    s.parse()
        .map_err(|err| format!("cannot parse bri: {:?}", err))
}

fn parse_rgb(r: &str, g: &str, b: &str) -> Result<(u8, u8, u8), String> {
    let r = r
        .parse()
        .map_err(|err| format!("cannot parse r: {:?}", err))?;
    let g = g
        .parse()
        .map_err(|err| format!("cannot parse g: {:?}", err))?;
    let b = b
        .parse()
        .map_err(|err| format!("cannot parse b: {:?}", err))?;
    Ok((r, g, b))
}
