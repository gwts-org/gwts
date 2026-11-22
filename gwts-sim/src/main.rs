use anyhow::Result;

type SignalId = usize;
type Tick = usize;

struct Signal {
    name: String,
    /// External signals are not set by simulation.
    /// They can be consts or inputs from a testbench.
    external: bool,
    state: bool,
    last_update: Tick,
}

/// Flip-Flop (FF)
/// This represents a 1-bit D flip-flop.
struct FF {
    prev_clock: bool,
    clock: SignalId,
    input: SignalId,
    output: SignalId,
    last_update: Tick,
}

/// Look-up Table (LUT)
/// This represents a 6-to-1 LUT.
struct LUT {
    inputs: [SignalId; 6],
    output: SignalId,
    truth_table: u64,
    last_update: Tick,
}

struct Simulation {
    signals: Vec<Signal>,
    ffs: Vec<FF>,
    luts: Vec<LUT>,
    tick: Tick,
}

impl Simulation {
    fn new() -> Self {
        Self {
            signals: Vec::new(),
            ffs: Vec::new(),
            luts: Vec::new(),
            tick: 0,
        }
    }

    fn update(&mut self) {
        self.tick += 1;

        for signal in &mut self.signals {
            // External signals are always up to date.
            if signal.external {
                signal.last_update = self.tick;
            }
        }

        let mut ffs_updated: usize = 0;
        let mut luts_updated: usize = 0;
        let mut prev_ffs_updated: usize = 0;
        let mut prev_luts_updated: usize = 0;

        // Loop until all blocks are simulated.
        // A block is considered simulatable if all of it's inputs have been updated this iteration.
        // Once it has been simulated, its output is marked as up to date.
        //
        // There is stall protection.
        loop {
            let ffs_done: bool = ffs_updated == self.ffs.len();
            let luts_done: bool = luts_updated == self.luts.len();

            if !ffs_done {
                for ff in &mut self.ffs {
                    // Skip if we've already simulated this ff.
                    if ff.last_update == self.tick {
                        continue;
                    }

                    // Require the clock signal be up to date.
                    let clock: &Signal = &self.signals[ff.clock];
                    if clock.last_update != self.tick {
                        continue;
                    }

                    // Require the input signal be up to date.
                    let input: &Signal = &self.signals[ff.input];
                    if input.last_update != self.tick {
                        continue;
                    }

                    // There are no more ways this ff cannot be simulated.
                    // So, mark it as being update to date.
                    ff.last_update = self.tick;
                    ffs_updated += 1;

                    // Figure out if the clock rose from that last tick.
                    let clock_rise = clock.state && (clock.state != ff.prev_clock);
                    ff.prev_clock = clock.state;

                    // Update the output.
                    let input_state = input.state;
                    let output: &mut Signal = &mut self.signals[ff.output];
                    output.last_update = self.tick;
                    if clock_rise {
                        output.state = input_state;
                    }
                }
            }

            if !luts_done {
                for lut in &mut self.luts {
                    // Skip if we've already simulated this lut.
                    if lut.last_update == self.tick {
                        continue;
                    }

                    // Require the input signals be up to date.
                    let inputs: Vec<&Signal> =
                        lut.inputs.iter().map(|id| &self.signals[*id]).collect();
                    for input in &inputs {
                        if input.last_update != self.tick {
                            continue;
                        }
                    }

                    // There are no more ways this lut cannot be simulated.
                    // So, mark it as being update to date.
                    lut.last_update = self.tick;
                    luts_updated += 1;

                    let input_state = inputs
                        .iter()
                        .rev()
                        .fold(0, |state, input| (state << 1) | (input.state as i32));

                    // Update output signal.
                    let next_state: bool = (lut.truth_table & (1 << input_state)) != 0;
                    let output = &mut self.signals[lut.output];
                    output.state = next_state;
                }
            }

            // Exit when all blocks have been simulated.
            if ffs_done && luts_done {
                break;
            }

            // Detect a stalled simulation.
            if ffs_updated < prev_ffs_updated {
                panic!("`ffs_updated` overflowed");
            }

            if luts_updated < prev_luts_updated {
                panic!("`luts_updated` overflowed");
            }

            let ffs_stalled: bool = prev_ffs_updated == ffs_updated && self.ffs.len() != 0;
            let luts_stalled: bool = prev_luts_updated == luts_updated && self.luts.len() != 0;
            prev_ffs_updated = ffs_updated;
            prev_luts_updated = luts_updated;

            if ffs_stalled && luts_stalled {
                panic!("the simulation stalled");
            }
        }
    }

    fn print_signals(&self) {
        for s in self.signals.iter().skip(2) {
            print!("{}={}, ", s.name, if s.state { 1 } else { 0 });
        }
        println!();
    }
}

fn main() -> Result<()> {
    let mut sim = Simulation::new();

    sim.signals.push(Signal {
        name: "#zero".into(),
        external: true,
        state: false,
        last_update: 1,
    });

    sim.signals.push(Signal {
        name: "#one".into(),
        external: true,
        state: true,
        last_update: 1,
    });

    let a_id = sim.signals.len();
    sim.signals.push(Signal {
        name: "a".into(),
        external: true,
        state: false,
        last_update: 0,
    });

    let b_id = sim.signals.len();
    sim.signals.push(Signal {
        name: "b".into(),
        external: true,
        state: false,
        last_update: 0,
    });

    let c_id = sim.signals.len();
    sim.signals.push(Signal {
        name: "c".into(),
        external: false,
        state: false,
        last_update: 0,
    });

    sim.luts.push(LUT {
        inputs: [a_id, b_id, 0, 0, 0, 0],
        output: c_id,
        truth_table: 0x6666666666666666,
        last_update: 0,
    });

    const INPUTS: [(bool, bool); 4] = [(false, false), (false, true), (true, false), (true, true)];

    for (a, b) in INPUTS {
        sim.signals[a_id].state = a;
        sim.signals[b_id].state = b;
        sim.update();
        sim.print_signals();
    }
    Ok(())
}
