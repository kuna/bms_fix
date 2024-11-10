use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Error, Read, Write},
};

use regex::Regex;

pub trait ChartMethods {
    // Read bms file. This will read every single line
    // of the bms.
    fn read(&mut self, path: &str) -> Result<(), Error>;

    // Fix given bms file.
    fn fix(&mut self) -> Result<(), String>;

    // Save given bms file.
    fn save(&self, path: &str) -> Result<(), Error>;

    // Set verbose level of the methods.
    fn set_verbose(&mut self, level: i32);

    // Whether the given chart is modified?
    fn is_modified(&self) -> bool;
}

pub struct Chart {
    // Raw string data of the given BMS file.
    lines: Vec<String>,

    // Metadata fields. Those fields are read-only as of now
    // and won't be affected.
    title: String,
    artist: String,
    genre: String,
    level: i32,

    // below are state of the reader, which is irrevant to the bms itself.

    // verbose level. level 0 does not print except error, level 1 prints info.
    verbose: i32,

    // Path of the current opened file. Read-only field.
    path: String,

    // Has the chart is modified?
    is_modified: bool,
}

impl ChartMethods for Chart {
    fn read(&mut self, path: &str) -> Result<(), Error> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        let lines: Vec<String> = buf
            .lines()
            .map(|l| l.expect("could not parse line"))
            .collect();

        // Now new file read complete, update data and clear the status
        self.lines = lines;
        self.path = path.to_string();
        self.is_modified = false;

        Ok(())
    }

    fn fix(&mut self) -> Result<(), String> {
        // Fix 02 channel
        let re = Regex::new(r"^#[0-9]{3}02:").unwrap();
        for (pos, v) in self.lines.iter_mut().enumerate() {
            if re.is_match(&v) {
                let args = v.split_once(":").unwrap();
                let val = args.1;
                let mut fixval: Option<&str> = None;
                if val.len() >= 10 {
                    // truncate if too long
                    fixval = Some(&val[..10]);
                }
                // If fix is not required then skip
                if fixval.is_none() {
                    continue;
                }
                let fixval = format!("{}:{}", args.0, fixval.unwrap());
                // log the fix if verbose
                if self.verbose > 0 {
                    println!(
                        "[INFO] File {}, Line {}, fixed: {} => {}",
                        self.path, pos, val, fixval
                    );
                }
                // update value
                *v = fixval;
                // update state
                self.is_modified = true;
            }
        }
        Ok(())
    }

    fn save(&self, path: &str) -> Result<(), Error> {
        let file = File::create(path)?;
        let mut buf = BufWriter::new(file);
        for line in self.lines.iter() {
            writeln!(&mut buf, "{}", line)?;
        }
        Ok(())
    }

    fn set_verbose(&mut self, level: i32) {
        self.verbose = level;
    }

    fn is_modified(&self) -> bool {
        return self.is_modified;
    }
}

impl Default for Chart {
    fn default() -> Self {
        Chart {
            lines: Vec::new(),
            title: String::new(),
            artist: String::new(),
            genre: String::new(),
            level: 0,
            verbose: 0,
            path: String::new(),
            is_modified: false,
        }
    }
}

#[cfg(test)]
mod test_chart {
    use std::path::PathBuf;

    use defer_lite::defer;

    use super::*;

    #[test]
    fn read_minimum_ok() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "minimum.bms",
        ]
        .iter()
        .collect();
        let testpath = d.to_str().unwrap();

        // Run and verify
        let mut chart = Chart::default();
        assert!(chart.read(testpath).is_ok());
        assert_eq!(testpath, chart.path);
    }

    #[test]
    fn read_err_notexist() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "no_such_file.bms",
        ]
        .iter()
        .collect();
        let testpath = d.to_str().unwrap();

        // Run and verify
        let mut chart = Chart::default();
        assert!(chart.read(testpath).is_err());
    }

    #[test]
    fn fix_ok() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "corrupt.bms",
        ]
        .iter()
        .collect();
        let testpath = d.to_str().unwrap();
        let mut chart = Chart::default();
        assert!(chart.read(testpath).is_ok());

        // Run and verify
        chart.set_verbose(1); // also wannt test verbose
        let r = chart.fix();
        assert!(r.is_ok());
        assert!(chart.is_modified());
        assert_eq!("#00002:0.99973333", chart.lines[16]);
    }

    #[test]
    fn fix_ok_nochange() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "minimum.bms",
        ]
        .iter()
        .collect();
        let testpath = d.to_str().unwrap();
        let mut chart = Chart::default();
        assert!(chart.read(testpath).is_ok());

        // Run and verify
        chart.set_verbose(1);
        let r = chart.fix();
        assert!(r.is_ok());
        assert!(!chart.is_modified());
    }

    #[test]
    fn save_ok() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "minimum.bms",
        ]
        .iter()
        .collect();
        let testpath = d.to_str().unwrap();
        let mut chart = Chart::default();
        assert!(chart.read(testpath).is_ok());

        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "test_output.bms",
        ]
        .iter()
        .collect();
        let testoutpath = d.to_str().unwrap();

        // Cleanup hook before save
        defer! {
            let _ = fs::remove_file(testoutpath);
        }

        // Run and verify
        let r = chart.save(testoutpath);
        assert!(r.is_ok());

        // Verify file existence
        let outfilemd = fs::metadata(testoutpath);
        assert!(outfilemd.is_ok());
        let outfilemd = outfilemd.unwrap();
        assert!(outfilemd.is_file());

        // Verify file content
        let mut chart2 = Chart::default();
        assert!(chart2.read(testoutpath).is_ok());
        assert_eq!(chart.lines.len(), chart2.lines.len());
        assert_eq!(chart.lines[10], chart2.lines[10]);
    }
}
