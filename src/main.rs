use std::{
    fmt::LowerHex,
    fs::File,
    io::{Read as _, Write},
    path::PathBuf,
    process::exit,
};

use sevenz_rust2::ArchiveReader;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    video: PathBuf,
}

fn main() -> testresult::TestResult {
    let args = Args::parse();
    let mut video = File::open(&args.video)?;
    let mut subtitles = args.video;
    subtitles.set_extension("txt");
    let mut subtitles = File::create_new(subtitles)?;
    let mut buf = vec![0; 10485760];
    video.read_exact(&mut buf)?;
    let digest = md5::compute(&buf);

    let hex_digest = hex::encode(*digest);
    let t_checksum = Checksum(&*digest);
    let url = format!(
        "https://napiprojekt.pl/unit_napisy/dl.php?l=PL&f={hex_digest}&t={t_checksum:x}&v=other&kolejka=false&nick=&pass=&napios=posix"
    );
    eprintln!("url {url}");
    let mut resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        eprintln!("bad: {resp:?}");
        exit(1);
    }

    let mut mem = vec![];
    std::io::copy(&mut resp, &mut mem)?;

    let src_reader = std::io::BufReader::new(std::io::Cursor::new(mem));
    let password = "iBlm8NTigvru0Jr0".into();
    let mut seven = ArchiveReader::new(src_reader, password)?;
    seven.for_each_entries(|_entry, reader| {
        let mut buf = vec![];
        std::io::copy(reader, &mut std::io::Cursor::new(&mut buf)).unwrap();
        let dec = encoding_rs::WINDOWS_1250.decode(&buf).0;
        subtitles.write_all(dec.as_bytes()).unwrap();
        Ok(true)
    })?;

    Ok(())
}

struct Checksum<'a>(&'a [u8]);

static TRIPLES: [(u8, u16, usize); 5] = [(0, 2, 14), (13, 2, 3), (16, 5, 6), (11, 4, 8), (5, 3, 2)];

impl LowerHex for Checksum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nibbles = self
            .0
            .iter()
            .flat_map(|b| [b / 16, b % 16])
            .collect::<Vec<_>>();

        for (add_i, mul_i, idx_i) in TRIPLES {
            let i = (add_i + nibbles[idx_i]) as usize;
            let s = u16::from_be_bytes([nibbles[i], nibbles[i + 1]]);
            write!(f, "{:x}", (s * mul_i) % 16)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[test]
    fn checksum() -> TestResult {
        let sum = "4b3d32b7700b3588531dd81db058eba9";
        let res = Checksum(&hex::decode(sum)?);
        assert_eq!(format!("{res:x}"), "00640");

        Ok(())
    }
}
