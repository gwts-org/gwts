use anyhow::Result;
use gwts_core::net_file::{Net, NetLUT, NetSignal};

const AND_TRUTH_TABLE: u64 = 0x8888888888888888;
const OR_TRUTH_TABLE: u64 = 0xEEEEEEEEEEEEEEEE;
const XOR_TRUTH_TABLE: u64 = 0x6666666666666666;

fn main() -> Result<()> {
    let net = Net {
        signals: vec![
            NetSignal {
                name: "#zero".into(),
            },
            NetSignal {
                name: "#one".into(),
            },
            NetSignal { name: "a".into() },
            NetSignal { name: "b".into() },
            NetSignal { name: "c".into() },
        ],
        ffs: vec![],
        luts: vec![NetLUT {
            input_indices: [2, 3, 0, 0, 0, 0],
            output_index: 4,
            truth_table: OR_TRUTH_TABLE,
        }],
    };

    std::fs::write("test.n", net.to_bytes())?;
    Ok(())
}
