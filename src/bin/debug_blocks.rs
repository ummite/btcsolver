use std::fs::File;
use std::io::Read;

fn main() {
    // Read first block file
    let mut file = File::open("Y:\\Bitcoin\\blocks\\blk00000.dat").expect("file not found");
    let mut buf = [0u8; 64];
    file.read_exact(&mut buf).expect("read error");

    println!("Raw bytes (first 64):");
    for i in (0..64).step_by(16) {
        let hex: String = buf[i..i+16].iter().map(|b| format!("{:02x} ", b)).collect();
        println!("  {:04}: {}", i, hex);
    }

    // Try BE key bytes
    let key_be = [0xb3, 0xa2, 0xcd, 0x52, 0x2d, 0xf3, 0xa0, 0x49];
    let mut deobf_be = buf.to_vec();
    for (i, b) in deobf_be.iter_mut().enumerate() {
        *b ^= key_be[i % 8];
    }

    println!("\nDeobfuscated (BE key, first 64):");
    for i in (0..64).step_by(16) {
        let hex: String = deobf_be[i..i+16].iter().map(|b| format!("{:02x} ", b)).collect();
        println!("  {:04}: {}", i, hex);
    }

    let magic_be = u32::from_le_bytes([deobf_be[0], deobf_be[1], deobf_be[2], deobf_be[3]]);
    println!("\nMagic BE (LE u32): {:08x} (expected d9b4bef9)", magic_be);

    let block_size_be = u32::from_le_bytes([deobf_be[4], deobf_be[5], deobf_be[6], deobf_be[7]]);
    println!("Block size BE: {} ({:.2} MB)", block_size_be, block_size_be as f64 / 1_048_576.0);

    // Also check raw magic
    let raw_magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    println!("\nRaw magic (LE u32): {:08x}", raw_magic);
}
