use {
    std::{rc::Rc, io::Read},
    linereader::LineReader
};

pub struct RcLineIterator<TRead: Read> {    
    line_reader: LineReader<TRead>,
    max_size: usize,
    buffer: Rc<String>,
    pending_incomplete: bool
}

impl<T: Read> RcLineIterator<T> {
    pub fn new(line_reader: LineReader<T>, max_size: usize) -> Self {
        Self {
            line_reader,
            max_size,
            buffer: Rc::new(String::new()),
            pending_incomplete: false
        }
    }
}

impl<TRead: Read> Iterator for RcLineIterator<TRead> {
    type Item = Result<Rc<String>, crate::Error<Rc<String>>>;
    fn next(&mut self) -> Option<Result<Rc<String>, crate::Error<Rc<String>>>> {        
        let buffer = &mut self.buffer;
        let max_size = self.max_size;
        let pending_incomplete = &mut self.pending_incomplete;
        
        self.line_reader.next_line().map(move |line| {
            let mut line = line?;
            let contains_delimiter = line.last() == Some(&b'\n');
            if contains_delimiter {
                line = &line[0..line.len() - 1];
                if line.last() == Some(&b'\r') {
                    line = &line[0..line.len() - 1];
                }
            }                    
            let owned = if let Some(r) = Rc::get_mut(buffer) {
                r.clear();
                r
            } else { 
                *buffer = Rc::new(String::with_capacity(line.len()));
                Rc::get_mut(buffer).unwrap()
            };
            let line_str = std::str::from_utf8(line)?;
            owned.push_str(line_str);
                    
            if max_size == buffer.len() {
                *pending_incomplete = true;
                Err(crate::Error::Incomplete(buffer.clone()))     
            } else if *pending_incomplete  {
                *pending_incomplete = false;
                Err(crate::Error::Incomplete(buffer.clone()))                                
            } else {
                Ok(buffer.clone())
            }         
        })
    }    
}