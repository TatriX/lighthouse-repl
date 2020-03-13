use lighthouse::{colors::*, state, HueBridge};
use prettytable::{cell, row, Table};
use std::thread::sleep;
use std::time::Duration;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    env_logger::init();

    let mut h = HueBridge::connect();
    let mut rl = Editor::<()>::new();

    let history_path = dirs::data_dir()
        .unwrap_or_else(|| "./".into())
        .with_file_name(".lhr_history");

    // Ignore missing history
    let _ = rl.load_history(&history_path);

    let mut t = term::stdout().unwrap();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if let Err(err) = process_line(&line, &h) {
                    t.fg(term::color::RED).unwrap();
                    println!("{}", err);
                    t.reset().unwrap();
                }
                rl.add_history_entry(line.as_str());
                {
                    use lighthouse::*;
                    let lights: std::collections::BTreeMap<u8, Light> = h
                        .request("lights", RequestType::Get, None)
                        .unwrap()
                        .json()
                        .unwrap();
                    let ids: Vec<u8> = lights.keys().cloned().collect();
                    let count = ids.len() as u8;

                    h.lights = LightCollection { lights, ids, count };
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
    rl.save_history(&history_path).unwrap();
}

fn process_line(line: &str, h: &HueBridge) -> Result<(), String> {
    let words: Vec<_> = line.split_whitespace().collect();
    Ok(match &words[..] {
        ["ls"] => list_lights(h),
        ["on", id] => light_set_on(parse_id(id)?, true, h),
        ["off", id] => light_set_on(parse_id(id)?, false, h),
        ["bri", id, bri] => light_set_bri(parse_id(id)?, parse_bri(bri)?, h),
        ["rgb", id, r, g, b] => light_set_color(parse_id(id)?, parse_rgb(r, g, b)?, h),
        ["play"] => play(h),
        _ => Err(format!("Unknown command: {}", line))?,
    })
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

fn light_set_color(id: u8, (r, g, b): (u8, u8, u8), h: &HueBridge) {
    h.state_by_ids(&[id], state!(xy: rgb_to_xy(r, g, b)))
        .unwrap();
}

fn light_set_on(id: u8, on: bool, h: &HueBridge) {
    if on {
        let bri = h.lights.lights[&id].state.bri;
        log::debug!("[light_set_on]: setting bri of {} to {}", id, bri);
        h.state_by_ids(&[id], state!(on: on, bri: bri)).unwrap();
    } else {
        h.state_by_ids(&[id], state!(on: false)).unwrap();
    }
}

fn light_set_bri(id: u8, bri: u8, h: &HueBridge) {
    log::debug!("[light_set_bri]: setting bri of {} to {}", id, bri);
    h.state_by_ids(&[id], state!(bri: bri)).unwrap();
}

fn list_lights(h: &HueBridge) {
    let mut table = Table::new();
    table.add_row(row!["id", "on", "name", "bri", "sat", "hue", "xy"]);
    for (id, light) in &h.lights.lights {
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

fn play(h: &HueBridge) {
    for i in (50..255).step_by(10) {
        let r = 255 - i;
        let g = i;
        let b = ((i as u64 * (i as u64 + 3) / 42 as u64) % 255) as u8;
        let dur = 3;
        println!("Setting bri to {}, color to {:?}", i, (r, g, b));
        h.state_by_ids(
            &[2, 4],
            state!(on: true, bri: i, xy: rgb_to_xy(r, g, b), transitiontime: dur),
        )
        .unwrap();

        sleep(Duration::from_millis(dur as u64 * 1000));
    }
}
