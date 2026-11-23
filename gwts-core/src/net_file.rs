use anyhow::Result;
use bytemuck::{Pod, Zeroable};

const MAGIC: u64 = u64::from_le_bytes(*b"GWTSNETS");
const VERSION: u64 = 0u64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct NetHeader {
    magic: u64,
    version: u64,
    ffs_offs: u64,
    ff_count: u64,
    luts_offs: u64,
    lut_count: u64,
    signals_offs: u64,
    signal_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct NetSignal {
    pub name: String,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct NetFF {
    pub clock_index: u64,
    pub input_index: u64,
    pub output_index: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct NetLUT {
    pub input_indices: [u64; 6],
    pub output_index: u64,
    pub truth_table: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Net {
    pub ffs: Vec<NetFF>,
    pub luts: Vec<NetLUT>,
    pub signals: Vec<NetSignal>,
}

impl Net {
    pub fn to_bytes(self) -> Vec<u8> {
        // TODO The signals need to be ordered.
        // This will make the file deterministic.
        // I don't know if that should be the responsibility of this code.
        // It probably should be done by the caller instead.

        // Prepare to build header later.
        let mut header = NetHeader::default();
        header.magic = MAGIC;
        header.version = VERSION;
        header.ff_count = self.ffs.len() as u64;
        header.lut_count = self.luts.len() as u64;
        header.signal_count = self.signals.len() as u64;

        // Build FF section.
        let mut ff_bytes: Vec<u8> = Vec::new();
        for ff in self.ffs {
            ff_bytes.extend_from_slice(bytemuck::bytes_of(&ff));
        }

        // Build LUT section.
        let mut lut_bytes: Vec<u8> = Vec::new();
        for lut in self.luts {
            lut_bytes.extend_from_slice(bytemuck::bytes_of(&lut));
        }

        // Build signals section.
        let mut signal_bytes: Vec<u8> = Vec::new();
        for signal in self.signals {
            let len: u16 = signal.name.len().try_into().expect("signal name too long");
            let len_bytes = len.to_le_bytes();
            let name_bytes = signal.name.into_bytes();

            signal_bytes.extend_from_slice(&len_bytes);
            signal_bytes.extend_from_slice(&name_bytes);
        }

        // Finish building header.
        let header_len: u64 = std::mem::size_of::<NetHeader>() as u64;
        let ffs_len: u64 = ff_bytes.len() as u64;
        let luts_len: u64 = lut_bytes.len() as u64;

        header.ffs_offs = header_len;
        header.luts_offs = header.ffs_offs + ffs_len;
        header.signals_offs = header.luts_offs + luts_len;

        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(bytemuck::bytes_of(&header));
        bytes.extend(&ff_bytes);
        bytes.extend(&lut_bytes);
        bytes.extend(&signal_bytes);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header_bytes = &bytes[0..std::mem::size_of::<NetHeader>()];
        let header: &NetHeader = bytemuck::from_bytes(header_bytes);

        if header.magic != MAGIC {
            panic!(
                "net file has an invalid magic value: 0x{:16X}",
                header.magic
            );
        }

        if header.version != VERSION {
            panic!("net file is an unsupported version: {}", header.version);
        }

        let ffs_offs: usize = header.ffs_offs as usize;
        let luts_offs: usize = header.luts_offs as usize;
        let signals_offs: usize = header.signals_offs as usize;

        let ffs_len: usize = luts_offs - ffs_offs;
        let luts_len: usize = signals_offs - luts_offs;
        let signals_len: usize = bytes.len() - signals_offs;

        let ff_bytes: &[u8] = &bytes[ffs_offs..(ffs_offs + ffs_len)];
        let lut_bytes: &[u8] = &bytes[luts_offs..(luts_offs + luts_len)];
        let signal_bytes: &[u8] = &bytes[signals_offs..(signals_offs + signals_len)];

        let ffs: Vec<NetFF> = bytemuck::cast_slice(ff_bytes).to_vec();
        let luts: Vec<NetLUT> = bytemuck::cast_slice(lut_bytes).to_vec();

        let mut signal_bytes = Vec::from(signal_bytes);
        let prev_len = signal_bytes.len();

        let mut signals: Vec<NetSignal> = Vec::new();
        while !signal_bytes.is_empty() {
            let len_bytes: &[u8] = &signal_bytes[..2];
            let len: usize =
                u16::from_le_bytes(len_bytes.try_into().expect("failed to get len bytes")) as usize;
            let signal_len: usize = 2 + len;

            // Grab name.
            let name_bytes: &[u8] = &signal_bytes[2..(2 + len)];
            let name: String = String::from_utf8_lossy(name_bytes).into_owned();
            signals.push(NetSignal { name });

            // Check for stalled parsing.
            signal_bytes.drain(..signal_len);
            if prev_len == signal_bytes.len() {
                panic!("net file signal parsing stalled");
            }
        }

        Ok(Self { ffs, luts, signals })
    }
}

pub fn foo() {
    println!("foo");
}
