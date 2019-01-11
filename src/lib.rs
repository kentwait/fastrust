#[macro_use]
extern crate cpython;
use cpython::{Python, PyResult, PyDict, ToPyObject};
use std::fs::File;
use std::io::prelude::*;

struct Seq {
    seq_id: String,
    description: String,
    sequence: String,
}

impl Seq {
    fn new(seq_id: &str, description: &str, sequence: &str) -> Seq {
        Seq {
            seq_id: seq_id.to_string(),
            description: description.to_string(),
            sequence: sequence.to_string(),
        }
    }
}

impl ToPyObject for Seq {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "seq_id", self.seq_id.as_str()).unwrap();
        dict.set_item(py, "description", self.description.as_str()).unwrap();
        dict.set_item(py, "sequence", self.sequence.as_str()).unwrap();

        dict
    }
}

fn parse_fasta(path: &str) -> Vec<Seq> {
    // Open path in read-only mode
    // Returns io::Result<File>
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(x) => panic!("couldn't read {}: {}", path, x),
    };

    // Load all contents into data
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => (),
        Err(x) => panic!("couldn't read {}: {}", path, x),       
    };
    
    let mut seq_list: Vec<Seq> = Vec::new();
    let mut seq_id: &str = "";
    let mut desc: &str = "";
    let mut sequence: Vec<String> = Vec::new();
    for line in data.lines() {
        if line.starts_with(">") {
            if sequence.len() > 0 {
                // Create a Seq struct and push to seq_list
                let seq = sequence.concat();
                let s = Seq::new(seq_id, desc, &seq);
                seq_list.push(s);

                // Clear contents
                sequence.clear();
            }
            // Process the ID line
            // Separate the ID field from the description field
            // Check whether a description field is present
            let line_contents: Vec<&str> = line.trim_right()
                                               .trim_start_matches('>')
                                               .splitn(2, ' ')
                                               .collect();
            if line_contents.len() == 2 {
                desc = line_contents[1];
            }
            seq_id = line_contents[0]
        } else {
            sequence.push(line.trim_right().to_string());
        }
    }
    
    seq_list 
}

// Wraps parse_fasta in order to be exportable
fn py_parse_fasta(_py: Python, path: &str) -> PyResult<Vec<Seq>> {
    let out = parse_fasta(path);
    Ok(out)
}

py_module_initializer!(fastrust, initfastrust, PyInit_fastrust, |py, m| { 
    m.add(py, "parse_fasta", py_fn!(py, 
        py_parse_fasta(path: &str)))?;

    Ok(())
});

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
