use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct NetsToml {
    signals: Vec<String>,
    luts: Vec<LutToml>,
}

#[derive(Deserialize, Debug)]
struct LutToml {
    inputs: Vec<String>,
    outputs: Vec<String>,
    table: HashMap<String, String>,
}

struct Signal {
    name: String,
    state: bool,
    last_update: usize,
}

struct Lut {
    // `inputs` and `outputs` hold signal indices.
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    // Each element in the truth table is indexed by treating the inputs as a binary representation of the truth table
    // index.
    table: Vec<Vec<bool>>,
}

impl Lut {
    fn solve(&self, sigs: &mut Vec<Signal>) {
        let input_state = self.inputs.iter().enumerate().fold(0, |acc, (pos, index)| {
            if sigs[*index].state {
                acc | (1 << pos)
            } else {
                acc
            }
        });

        for (i, state) in self.table[input_state].iter().enumerate() {
            let index: usize = self.outputs[i];
            let signal: &mut Signal = &mut sigs[index];
            signal.state = *state;
        }
    }
}

struct Context {
    sigs: Vec<Signal>,
    luts: Vec<Lut>,
}

fn main() -> Result<()> {
    let nets_path = std::env::args()
        .nth(1)
        .expect("Usage: gtws-sim <net> <inputs>");
    let nets_text = std::fs::read_to_string(nets_path)?;
    let nets_toml: NetsToml = toml::from_str(&nets_text)?;

    let mut sigs: Vec<Signal> = nets_toml
        .signals
        .iter()
        .map(|name| Signal {
            name: name.to_owned(),
            state: false,
            last_update: 0,
        })
        .collect();

    let luts: Vec<Lut> = nets_toml
        .luts
        .iter()
        .map(|lut| {
            let inputs: Vec<usize> = lut
                .inputs
                .iter()
                .map(|name| {
                    sigs.iter().position(|sig| sig.name == *name).expect(
                        format!("LUT outputs reference unknown signal ({:?}).", name).as_str(),
                    )
                })
                .collect();

            let outputs: Vec<usize> = lut
                .outputs
                .iter()
                .map(|name| {
                    sigs.iter().position(|sig| sig.name == *name).expect(
                        format!("LUT outputs reference unknown signal ({:?}).", name).as_str(),
                    )
                })
                .collect();

            if lut.table.len() != 2usize.pow(lut.inputs.len() as u32) {
                panic!("LUT truth table must fit exactly 2^N where N = number of inputs.");
            }

            let table_tmp: Vec<(usize, Vec<bool>)> = lut
                .table
                .iter()
                .map(|(input_state, output_state)| {
                    let index = usize::from_str_radix(input_state, 16).unwrap();
                    let state_int = usize::from_str_radix(output_state, 16).unwrap();

                    let state: Vec<bool> = (0..outputs.len())
                        .map(|bit| (state_int & (1usize << bit)) != 0)
                        .collect();

                    (index, state)
                })
                .collect();

            let mut table = vec![Vec::new(); lut.table.len()];
            for (index, state) in table_tmp {
                table[index] = state;
            }

            Lut {
                inputs,
                outputs,
                table,
            }
        })
        .collect();

    let inputs_path = std::env::args()
        .nth(2)
        .expect("Usage: gtws-sim <net> <inputs>");
    let inputs_text = std::fs::read_to_string(inputs_path)?;

    for (t, line) in inputs_text.lines().enumerate() {
        let assignments = line.split(',').map(|s| s.trim()).map(|s| {
            let mut s = s.split('=');
            (
                s.nth(0).expect("expected left side operand"),
                s.nth(0).expect("expected right side operand") == "1",
            )
        });

        for (name, value) in assignments {
            let signal: &mut Signal = sigs.iter_mut().find(|s| s.name == name).unwrap();
            signal.state = value;
        }

        for lut in luts.iter() {
            lut.solve(&mut sigs);
        }

        print!("t={}: ", t);
        for sig in sigs.iter() {
            print!("{}={} ", sig.name, if sig.state { 1 } else { 0 });
        }
        println!();
    }

    Ok(())
}
