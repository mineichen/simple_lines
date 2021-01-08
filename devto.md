
It was great fun solving last year's [Advent of Code](https://adventofcode.com/2020) using Rust. 
The problems were all approachable and got me thinking about new algorithms to solve them. A small part of the resulting code ended up in a "utils" crate to be accessible from every sub project in the workspace. The utils-crate included a struct called LineReaderIterator with the following signature:
```rust
pub struct LineReaderIterator<T, TFn: FnMut(&str) -> Result<T, Error>, TRead: Read> {
    reader: BufReader<TRead>,
    buffer: String,
    mapper: TFn,
}
```

But why would you write your own iterator when there is `std::io::BufRead::lines()` in the standard library? When I looked at the lines()-signature, it resulted in a minor hickup: It returns a fresh `String` for each line instead of reusing a common buffer after the line was processed. Of course this never really impacted performance with such small input files, but as the entire AoC is about fun, I wanted to do that without this additional allocation.

The above iterator was able to parse each line into an appropriate struct. However, it wasn't perfect. When the first puzzle with multiple input sections arrived, I ended up using `std::BufRead::lines()` again, because there was no way to stop parsing and process the rest of the lines with another reducer. I thought that there must be a way to implement an `Iterator<Item=&mut str>`. After all, `slice::IterMut` also returns mutable references to it's data with an iterator. 

Unfortunately it turned out you can't, because the signature of iterator makes it impossible to return a reference which only lives until the next call to `next()`.
```rust
next(&mut self) -> Option<Self::Item>;
```

But how is `Slice::iter_mut()` doing it? To my surprise, all references retrieved by the iterator could outlive the result of the next item. This is sound, because it never &mut references the same field twice.
```rust
let mut data = [1,1];
let mut data_iter = data.iter_mut();
let first = data_iter.next().unwrap();
let second = data_iter.next().unwrap();
assert_eq!(first, second); 
```

At this point I lost hope to be able to implement my Iterator<&mut str> in a sound way. Even thought I could reuse the buffer with unsafe, the implementation would be unsound as there is no way to assure that the reference from the previous `next()` isn't used anymore. At least not without runtime checks. But is a runtime check really worse than allocating memory over and over again? What if we try to reuse a `std::rc::Rc` if the consumer doesn't keep any reference to the previous value? In fact, this seems to work just fine:
```rust
pub struct RcLineReaderIterator<TRead: Read> {
    reader: BufReader<TRead>,
    buffer: Rc<String>
}

impl<TRead: Read> RcLineReaderIterator<TRead> {
    fn new(reader: TRead) -> Self {
        Self { reader: BufReader::new(reader), buffer: Rc::new(String::new()) }
    }
}

impl<TRead: Read> Iterator for RcLineReaderIterator<TRead> {
    type Item = Result<std::rc::Rc<String>, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = if let Some(r) = Rc::get_mut(&mut self.buffer) {
            println!("Reuse");
            r.clear();
            r
        } else {
            println!("New Buffer");            
            self.buffer = Rc::new(String::new());
            Rc::get_mut(&mut self.buffer).unwrap()
        };
        match self.reader.read_line(buf) {
            Ok(n) if n > 0 => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(self.buffer.clone()))
            },
            Ok(_) => None,
            Err(e) => Some(Err(e.into())),
        }
    }
}
```
Here you see a working example:
```rust
#[test]
fn test_keep_ref() {
    let a = "Foo\nFoo\na\nbb\nccc\n";
    let cursor = std::io::Cursor::new(a);
    let mut iter = RcLineReaderIterator::new(cursor);
    {
        assert_eq!(
            iter.next().unwrap().unwrap(), 
            iter.next().unwrap().unwrap()
        );
    }
    assert_eq!(6, iter.filter_map(|i| i.map(|i| i.len()).ok()).sum::<usize>());
}
```
Outputs:
```
running 1 test
Reuse
New Buffer
Reuse
Reuse
Reuse
Reuse
test tests::test_keep_ref ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
```
This pattern could be used in other places too, e.g. within a map() function.

# Line as an attack vector
A slightly bigger hickup occured when I've read the docs of `BufRead::read_line()`. 
> This function is blocking and should be used carefully: it is possible for an attacker to continuously send bytes without ever sending a newline or EOF.

It is not very rust-like to just mention such problems in the docs. I'd prefer to have an API which makes you aware of that. While writing this blog post I decided to write a small library avaliable at [simple_lines](https://crates.io/crates/simple_lines), which returns an `Result<Rc<String>, Error>` for each line:
```rust
pub enum Error {
    Io(std::io::Error),
    Encoding(std::str::Utf8Error),
    Incomplete(Rc<String>), // unfinished lines with len() > buffer_size
}
```
Of course, the actual implementation doesn't use `std::BufRead::read_line()`, but an alternative crate in the background. A benchmark counting all charracters showed, that my implementation is much faster (34.4 MB in 49.312ms vs 127.73ms with `std::io::BufReader::lines()`) and is much safer to be used in production.

# A quick look into the future
I was curious if GATs could be used to address this issue and finally allow borrowing data until the next call to `Iterator::next()`. As the feature is not yet stable, it is probably still too early to propose extensions to it, but I'd appreciate to have a special lifetime (e.g. '_ or 'self) to be used in GATs pointing to the self-arguments lifetime... This way, the Iterator-Trait could be left unchanged and thus stay backwards compatible but capable of returning references to the iterator itself. But as you might have noticed, I left my area of expertice already. Looking forward to hear your opinion about that. 
