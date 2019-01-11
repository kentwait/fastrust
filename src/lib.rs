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

// TODO: Change output to Some<Vec<Seq>> to account for error
fn read_fasta(path: &str) -> Vec<Seq> {
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
    if sequence.len() > 0 {
        // Create a Seq struct and push to seq_list
        let seq = sequence.concat();
        let s = Seq::new(seq_id, desc, &seq);
        seq_list.push(s);

        // Clear contents
    }
    
    seq_list 
}

// Wraps read_fasta in order to be exportable
fn py_read_fasta(_py: Python, path: &str) -> PyResult<Vec<Seq>> {
    let out = read_fasta(path);
    Ok(out)
}

fn write_fastq(sequences: Vec<Seq>, path: &str, linewidth: i32) -> i32 {
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(x) => panic!("couldn't create {}: {}", path, x),
    };
    let mut str_list: Vec<String> = Vec::new();
    for seq in sequences {
        {
            let mut line = format!(">{}", seq.seq_id);
            
            if seq.description.len() > 0 {
                line = format!("{} {}", line, seq.description);
            }
            str_list.push(line)
        }

        if linewidth == -1 {
            str_list.push(seq.sequence.to_string()); 
        } else if linewidth > 0 {
            let sub_seqs = seq.sequence.as_bytes()
                .chunks(linewidth as usize)
                .map(|s| unsafe { ::std::str::from_utf8_unchecked(s).to_string() })
                .collect::<Vec<_>>();
            str_list.extend(sub_seqs);
        } else {
            panic!("line width must be > 0 or -1")
        }
    }
    let data = str_list.join("\n");
    match file.write_all(data.as_bytes()) {
        Ok(_) => (),
        Err(x) => panic!("couldn't write to {}: {}", path, x)
    }

    1
}

// Wraps write_fastq in order to be exportable
fn py_write_fastq(_py: Python, sequences: Vec<PyDict>, path: &str, linewidth: i32) -> PyResult<i32> {
    let mut sequences_: Vec<Seq> = Vec::new();
    for dict in sequences {
        let s = Seq {
            seq_id: match dict.get_item(_py, "seq_id") {
                Some(v) => format!("{}", v),
                None => panic!("seq_id key not found!"),
            },
            description: match dict.get_item(_py, "description") {
                Some(v) => format!("{}", v),
                None => panic!("description key not found!"),
            },
            sequence: match dict.get_item(_py, "sequence") {
                Some(v) => format!("{}", v),
                None => panic!("sequence key not found!"),
            },
        };
        sequences_.push(s);
    }
    let out = write_fastq(sequences_, path, linewidth);
    Ok(out)
}

py_module_initializer!(fastrust, initfastrust, PyInit_fastrust, |py, m| { 
    m.add(py, "read_fasta", py_fn!(py, py_read_fasta(path: &str)))?;
    m.add(py, "write_fastq", py_fn!(py, 
        py_write_fastq(sequences: Vec<PyDict>, path: &str, linewidth: i32)))?;

    Ok(())
});

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
