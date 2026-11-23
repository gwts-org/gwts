use anyhow::Result;
use gwts_core::{
    net_file::{Net, NetFF, NetLUT, NetSignal},
    tokenizer::{Token, TokenKind},
};

const AND_TRUTH_TABLE: u64 = 0x8;
const OR_TRUTH_TABLE: u64 = 0xE;
const XOR_TRUTH_TABLE: u64 = 0x6;

const USAGE: &'static str = "usage: gwts-syn <input> <output>";

fn main() -> Result<()> {
    let input_path = std::env::args().nth(1).expect(USAGE);
    let output_path = std::env::args().nth(2).expect(USAGE);
    let net = load_net(input_path.as_str())?;
    std::fs::write(output_path, net.to_bytes())?;
    Ok(())
}

// TODO This is just to help get things moving.
//
// This function parses an intermidate net representation.
// It's kind of like assembly.
// There are three "instructions": "ff", "lut", and "signal".
//
// "ff" defines a flip-flop.
// "ff <clock> <input> <output>"
//
// "lut" defines a look-up table.
// "lut <input0> <input1> <input2> <input3> <input4> <input5> <output> <truth-table>"
// The truth table is represented by a hexidecimal integer. It holds 64 bits where each bit represents an output. Which
// output is taken is determined by or'ng the inputs together which each "inputN" shifted to the right by "N".
//
// "signal" declares a signal.
// "signal <name>"
fn load_net(path: &str) -> Result<Net> {
    let mut ffs: Vec<(String, String, String)> = Vec::new();
    let mut luts: Vec<(Vec<String>, String, String)> = Vec::new();
    let mut signals: Vec<String> = Vec::new();

    let net_ir_text = std::fs::read_to_string(path)?;
    let mut tokens = gwts_core::tokenizer::tokens(net_ir_text.as_str())?;
    let mut prev_count = tokens.len();
    while !tokens.is_empty() {
        let first_token = tokens.remove(0);

        match first_token.kind {
            TokenKind::Alphanumeric(token_content) => match token_content.to_lowercase().as_str() {
                "ff" => {
                    if tokens.len() < 3 {
                        anyhow::bail!("expected 3 arguments to ff definition");
                    }

                    let clock: Token = tokens.remove(0);
                    let TokenKind::Alphanumeric(clock) = clock.kind else {
                        anyhow::bail!("expected clock in ff definition to be alphanumeric");
                    };

                    let input: Token = tokens.remove(0);
                    let TokenKind::Alphanumeric(input) = input.kind else {
                        anyhow::bail!("expected input in ff definition to be alphanumeric");
                    };

                    let output: Token = tokens.remove(0);
                    let TokenKind::Alphanumeric(output) = output.kind else {
                        anyhow::bail!("expected output in ff definition to be alphanumeric");
                    };

                    ffs.push((clock, input, output));
                }

                "lut" => {
                    if tokens.len() < 8 {
                        anyhow::bail!("expected 8 arguments to ff definition");
                    }

                    let inputs: Vec<Token> = tokens.drain(..6).collect();
                    let inputs: Vec<String> = inputs
                        .iter()
                        .map(|token| {
                            let TokenKind::Alphanumeric(name) = token.kind.to_owned() else {
                                panic!("expected inputs in ff definition to all be alphanumeric");
                            };
                            name
                        })
                        .collect();

                    let output: Token = tokens.remove(0);
                    let TokenKind::Alphanumeric(output) = output.kind else {
                        anyhow::bail!("expected output in ff definition to be alphanumeric");
                    };

                    let truth_table: Token = tokens.remove(0);
                    let TokenKind::Numeric(truth_table) = truth_table.kind else {
                        anyhow::bail!("expected truth table in ff definition to be numeric");
                    };

                    luts.push((inputs, output, truth_table));
                }

                "signal" => {
                    let TokenKind::Alphanumeric(name) = tokens.remove(0).kind else {
                        anyhow::bail!("expected signal name");
                    };

                    signals.push(name);
                }

                _ => anyhow::bail!(
                    "unexpected alphanumeric token in stream: {:?}",
                    token_content
                ),
            },

            _ => anyhow::bail!("unexpected token in stream: {:?}", first_token.kind),
        }

        if prev_count == tokens.len() {
            anyhow::bail!("parsing net IR stalled");
        }
        prev_count = tokens.len();
    }

    signals.sort();
    let signals: Vec<NetSignal> = signals
        .iter()
        .map(|name| NetSignal {
            name: name.to_owned(),
        })
        .collect();

    let ffs: Vec<NetFF> = ffs
        .iter()
        .map(|(clock_name, input_name, output_name)| NetFF {
            clock_index: find_signal(&signals, clock_name),
            input_index: find_signal(&signals, input_name),
            output_index: find_signal(&signals, output_name),
        })
        .collect();

    let luts: Vec<NetLUT> = luts
        .iter()
        .map(|(input_names, output_name, truth_table_text)| {
            let truth_table: u64 = u64::from_str_radix(truth_table_text, 16)
                .expect("truth table must be hexidecimal and fit in a u64");
            NetLUT {
                input_indices: input_names
                    .iter()
                    .map(|name| find_signal(&signals, name))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                output_index: find_signal(&signals, output_name),
                truth_table,
            }
        })
        .collect();

    Ok(Net { ffs, luts, signals })
}

fn find_signal(signals: &Vec<NetSignal>, name: &str) -> u64 {
    signals
        .iter()
        .position(|signal| signal.name == name)
        .expect(format!("cannot find signal: {:?}", name).as_str()) as u64
}
