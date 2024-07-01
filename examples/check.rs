use std::fs::File;
use std::io::Read;

fn main() -> anyhow::Result<()> {
    let mut ex_bin = File::open("ex.bin")?;
    let mut buf = Vec::new();
    ex_bin.read_to_end(&mut buf)?;
    for i in 0..10000 {
        let string = String::from_(&buf[i..]);
        if string.contains("magnet") {
            dbg!(i);
            dbg!(string);
            break;
        }
    }


    Ok(())
}