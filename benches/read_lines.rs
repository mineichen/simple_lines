use  {
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    simple_lines::ReadExt,
    std::io::{Cursor, BufRead}
};

fn compare_bufread_lines(c: &mut Criterion) {
    const FILE : &str = "Dickens_Charles_Pickwick_Papers.xml";
    let input = std::fs::read_to_string(FILE).expect("Download input from http://hur.st/Dickens_Charles_Pickwick_Papers.xml.xz and extract it into the project root");
    c.bench_function("simple_lines::LineIterable::lines_rc()", |b| b.iter(|| {
        assert_eq!(33532728, Cursor::new(black_box(input.clone()))
            .lines_rc()
            .filter_map(Result::ok)
            .fold(0, |acc, n| acc + n.len()))
    }));
    c.bench_function("std::BufReader::lines()", |b| b.iter(|| {
        assert_eq!(33532728, std::io::BufReader::new(Cursor::new(black_box(input.clone())))
            .lines()
            .filter_map(Result::ok)
            .fold(0, |acc, n| acc + n.len()))
    }));
    c.bench_function("linereader::LineReader().next_line()", |b| b.iter(|| {
        let mut cnt = 0;
        let mut iter = linereader::LineReader::new(Cursor::new(black_box(input.clone())));
        while let Some(x) = iter.next_line() {
            if let Ok(mut buf) = x {
                if buf.last() == Some(&b'\n') {
                    buf = &buf[0..buf.len() - 1];
                    if buf.last() == Some(&b'\r') {
                        buf = &buf[0..buf.len() - 1];
                    }
                }
                if let Ok(input) = std::str::from_utf8(buf) {
                    cnt += input.len();
                }
            }
        }
        
        assert_eq!(33532728, cnt);
    }));
}

criterion_group!(benches, compare_bufread_lines);
criterion_main!(benches);
