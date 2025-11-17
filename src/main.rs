use std::{io::Read as _, process::exit};

use sevenz_rust2::ArchiveReader;

fn main() -> testresult::TestResult {
    let mut ctx = md5::Context::new();
    let mut file = std::io::stdin();
    for _chunk in 0..10 {
        let mut data = [0; 1048576];
        file.read(&mut data[..])?;
        ctx.consume(data);
    }
    let hex_digest = hex::encode(*ctx.finalize());
    let t_checksum = calc_checksum(&hex_digest);
    let url = format!(
        "https://napiprojekt.pl/unit_napisy/dl.php?l=PL&f={hex_digest}&t={t_checksum}&v=other&kolejka=false&nick=&pass=&napios=posix"
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
        println!("{dec}");
        Ok(true)
    })?;

    Ok(())
}

fn calc_checksum(sum: &str) -> String {
    use std::fmt::Write;

    let idx = [14, 3, 6, 8, 2];
    let mul = [2, 2, 5, 4, 3];
    let add = [0, 13, 16, 11, 5];

    let mut out = String::new();

    for p in 0..5 {
        let n = add[p];
        let a = mul[p];
        let p = idx[p];
        //eprintln!("n {n} a {a} p {p}");
        let i = sum.chars().nth(p).unwrap().to_digit(16).unwrap();
        let i = (n + i) as usize;
        let s = sum.chars().nth(i).unwrap().to_digit(16).unwrap() * 16
            + sum.chars().nth(i + 1).unwrap().to_digit(16).unwrap();
        let y = s * a;
        //eprintln!("i {i} s {s} y {y}");
        let y = format!("{y:x}");
        write!(out, "{}", y.chars().last().unwrap()).unwrap();
        //eprintln!("napisum {out}");
    }

    out
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::calc_checksum;

    #[test]
    fn checksum() -> TestResult {
        let sum = "4b3d32b7700b3588531dd81db058eba9";
        let res = calc_checksum(&sum);
        assert_eq!(res, "00640");

        Ok(())
    }
}
