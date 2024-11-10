mod bms;

use bms::chart::ChartMethods;
use std::{
    env,
    error::Error,
    fs::{metadata, read_dir},
};

fn append_file_lists(path: &str, res: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    let md = metadata(path)?;
    if md.is_file() {
        res.push(path.to_string());
        return Ok(());
    } else if md.is_dir() {
        let paths = read_dir(path).unwrap();
        for path in paths {
            // Note: can't use
            // let subpath = path.unwrap().path().to_str().unwrap_or_default();
            // because of "temporary value dropped while borrowed"
            let pathbuf = path.unwrap().path();
            let subpath = pathbuf.to_str().unwrap_or_default();
            append_file_lists(subpath, res)?;
        }
        return Ok(());
    }
    return Err(format!("invalid file type, path: {}", path))?;
}

fn get_file_lists(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut v = Vec::new();
    append_file_lists(path, &mut v)?;
    Ok(v)
}

fn main() {
    println!("Hello, world!");

    // gather file list first
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Parameter: [file or folder to inspect]");
    }
    let paths = get_file_lists(&args[0]).unwrap();

    // filter paths.
    // if path does not ends with bms/bml/bme/bmx, skip processing the file.
    let paths: Vec<&String> = paths
        .iter()
        .filter(|s| {
            s.ends_with(".bms") || s.ends_with(".bms") || s.ends_with(".bms") || s.ends_with(".bms")
        })
        .collect();

    println!("Found {} files to process.", paths.len());

    // For each files do fixation
    // TODO: use threadpool with workers
    let dryrun = true;
    // let workers = 8;
    for path in paths {
        println!("Processing {} ...", path);
        let mut chart = bms::chart::Chart::default();
        chart.set_verbose(1);
        chart.read(&path).unwrap();
        chart.fix().unwrap();
        if !dryrun && chart.is_modified() {
            println!("Saving {} ...", path);
            chart.save(&path).unwrap();
        }
    }
}

#[cfg(test)]
mod test_main {
    use std::path::PathBuf;

    use crate::get_file_lists;

    #[test]
    fn get_file_lists_single_file() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "minimum.bms",
        ]
        .iter()
        .collect();

        // Run and verify
        let r = get_file_lists(d.to_str().unwrap());
        assert!(r.is_ok());
        let v = r.unwrap();
        assert_eq!(1, v.len());
        assert!(v[0].ends_with("minimum.bms"));
    }

    #[test]
    fn get_file_lists_non_exist() {
        // Setup test
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "resources",
            "test",
            "such_file_should_not_exist.bms",
        ]
        .iter()
        .collect();

        // Run and verify
        let r = get_file_lists(d.to_str().unwrap());
        assert!(r.is_err())
    }

    #[test]
    fn get_file_lists_directory() {
        // Setup test
        let d: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "test"]
            .iter()
            .collect();

        // Run and verify
        let r = get_file_lists(d.to_str().unwrap());
        assert!(r.is_ok());
        let mut v = r.unwrap();
        assert_eq!(3, v.len());
        // sort in advance to keep the result consistent
        v.sort();
        assert!(v[1].ends_with("/minimum.bms"));
        assert!(v[2].ends_with("/subdir/empty.bms"));
    }
}
