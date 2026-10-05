// SPDX-License-Identifier: AGPL-3.0-or-later
// Test-only renderer adapter: composite alpha over white, then integer BT.601 luma.
use std::{
    fs,
    io::{BufWriter, Write},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: normalize_fixture INPUT.png OUTPUT.pgm".into());
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(fs::read(&args[1])?));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut data = vec![0; reader.output_buffer_size().ok_or("buffer overflow")?];
    let info = reader.next_frame(&mut data)?;
    if info.width as usize * info.height as usize > audiveris_native::MAX_PIXELS {
        return Err("fixture exceeds pixel cap".into());
    }
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err("unexpected color".into()),
    };
    let mut pixels = Vec::new();
    for p in data[..info.buffer_size()].chunks_exact(channels) {
        let (r, g, b, a) = match channels {
            1 => (p[0], p[0], p[0], 255),
            2 => (p[0], p[0], p[0], p[1]),
            3 => (p[0], p[1], p[2], 255),
            _ => (p[0], p[1], p[2], p[3]),
        };
        let over = |c: u8| (u32::from(c) * u32::from(a) + 255 * (255 - u32::from(a)) + 127) / 255;
        pixels.push(((299 * over(r) + 587 * over(g) + 114 * over(b) + 500) / 1000) as u8);
    }
    let mut out = BufWriter::new(
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    writeln!(out, "P5\n{} {}\n255", info.width, info.height)?;
    out.write_all(&pixels)?;
    out.flush()?;
    println!(
        "{}x{} {:?} -> grayscale, no resize",
        info.width, info.height, info.color_type
    );
    Ok(())
}
