use std::io;

#[allow(dead_code)]
pub trait Terminal {
    fn read_line(&mut self) -> io::Result<String>;
    fn write(&mut self, text: &str) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
}

pub struct StdTerminal;

impl Terminal for StdTerminal {
    fn read_line(&mut self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }

    fn write(&mut self, text: &str) -> io::Result<()> {
        print!("{text}");
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        use io::Write;
        io::stdout().flush()
    }
}

#[cfg(test)]
pub struct MockTerminal {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    input_index: usize,
}

#[cfg(test)]
impl MockTerminal {
    #[allow(dead_code)]
    pub fn new(inputs: Vec<String>) -> Self {
        Self {
            inputs,
            outputs: Vec::new(),
            input_index: 0,
        }
    }
}

#[cfg(test)]
impl Terminal for MockTerminal {
    fn read_line(&mut self) -> io::Result<String> {
        if self.input_index < self.inputs.len() {
            let input = self.inputs[self.input_index].clone();
            self.input_index += 1;
            Ok(input)
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "No more inputs",
            ))
        }
    }

    fn write(&mut self, text: &str) -> io::Result<()> {
        self.outputs.push(text.to_string());
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
