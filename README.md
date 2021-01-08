# Simple and secure line iterators
Simple line iterator which prevents OutOfMemory if an attacker inputs very long sequences without a delimiter by applying a max_capacity. The implementation reuses the last Rc<String> on calling next() if it isnt used anymore.

It currently uses the linebuffer library under the hood but provides a much simpler interface with fewer pitfalls:
 - Incomplete lines result in Err(Incomplete<Rc<String>>) to force users to think about this scenario
 - Ok variant should be compatible with std::io::BufReader beside wrapping in Rc
 - Invalid UTF8 results in Err(Encoding)


## cargo bench (version 0.0.1)

Tests performed using ['Dickens_Charles_Pickwick_Papers.xml'](http://hur.st/Dickens_Charles_Pickwick_Papers.xml.xz),
which is 34.4 MB big and 845k lines long.

### Bacbook Pro 2019 2.6 GHz 6-Core Intel Core i7
```
simple_lines::LineIterable::lines_rc()
time:   [48.811 ms **49.312** ms 49.827 ms
change: [+0.2107% +1.4272% +2.6542%] (p = 0.02 < 0.05)
Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking std::BufReader::lines(): Warming up for 3.0000 s
std::BufReader::lines() 
time:   [125.78 ms **127.73** ms 129.82 ms]                                    
change: [-3.1501% -1.3464% +0.6779%] (p = 0.17 > 0.05)
No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) high mild
  12 (12.00%) high severe

linereader::LineReader().next_line() 
time:   [43.261 ms **43.480 ms** 43.714 ms]
change: [+4.2718% +5.0450% +5.8169%] (p = 0.00 < 0.05)
Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
```