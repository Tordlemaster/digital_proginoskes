//Download the Tycho-2 data from https://cdsarc.u-strasbg.fr/viz-bin/Cat?I/259#/browse
//Unzip it by calling tar with std::process
//process the data culling out stars below a specified magnitude
//and then create a new file with either the quadtree format or the GPU instanced points format
//(icosahedron spherical quadtree or cube spherical quadtree) (cube might be easier)
//write it with serde???

//Attempt to open and read from data file and if fail then download it

//Each cell of the quadtree should have an average color value so that
//if there are many stars packed into 1 pixel the quadtree navigation can stop at that point and just return the precalculated value

use std::io::{BufRead, BufReader, BufWriter};
use std::process::Command;
use std::{fs, io::Write};
use std::fs::{canonicalize, File, OpenOptions};
use std::path::{Path, PathBuf};

use curl::easy::{Easy, WriteError};

mod spherical_quadtree;


const TYCHO_2_URL: &str = "https://cdsarc.u-strasbg.fr/viz-bin/nph-Cat/tar.gz?I/259";

fn download_data() {
    fs::create_dir_all("./data/download").unwrap();
    let file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("./data/download/I_259.tar.gz")
        .unwrap();
    let mut buf = BufWriter::new(file);

    let mut easy = Easy::new();
    easy.url(TYCHO_2_URL).unwrap();
    easy.write_function(move |data| {
        match buf.write(data) {
            Ok(n) => Ok(n),
            Err(_) => Err(WriteError::Pause)
        }
    }).expect("File write failed");
    easy.perform().expect("Download failed");
    println!("Successfully downloaded data");
}

fn extract_data() {
    let _ = fs::remove_dir_all("./data/download/extract");
    let _ = fs::remove_dir_all("./data/download/tmp");
    let _ = fs::create_dir("./data/download/tmp");
    let _ = fs::create_dir("./data/download/extract");

    //Extract files from original archive
    let status = Command::new("tar")
        .args(["-xf", "./data/download/I_259.tar.gz", "-C", "./data/download/tmp"])
        .status()
        .expect("Failed to unpack data");
    println!("Finished unpacking archive with status {status}");

    //Uncompress data files
    const DATA_FILES: [&str; 2] = ["index.dat.gz", "suppl_1.dat.gz"];
    let dest_path = canonicalize("./data/download/extract")
        .unwrap();
    let dest = format!("-o{}", dest_path.to_str().unwrap());

    for s in DATA_FILES {
        let mut fp: String = String::from("./data/download/tmp/");
        fp.push_str(s);
        
        /*let status = Command::new("tar")
            .args(["-xf", fp.as_str(), "-C", "./data/download/extract"])
            .status()
            .expect(format!("Failed to unpack {}", s).as_str());*/
        let status = Command::new("7z")
            .args(["x", fp.as_str(), dest.as_str()])
            .current_dir(".")
            .status()
            .unwrap();
        println!("Finished unpacking archive with status {status}");
    }

    for i in 0..=19 {
        let file = format!("tyc2.dat.{:02}.gz", i);
        let fp: String = format!("./data/download/tmp/{}", file);
        //let fpath = canonicalize(fp).unwrap();
        
        /*let status = Command::new("tar")
            .args(["-xf", fp.as_str(), "-C", "./data/download/extract"])
            .status()
            .expect(format!("Failed to unpack {}", file).as_str());*/
        let status = Command::new("7z")
            .args(["x", fp.as_str(), dest.as_str()])
            .current_dir(".")
            .status()
            .unwrap();
        println!("Finished unpacking archive with status {status}");
    }
    
    //Cleanup
    let _ = fs::remove_dir_all("./data/download/tmp");
}

//Have one thread that processes the file and appends the data to a structure and another that constructs the quadtree
fn generate_cpu_quadtree() {
    for i in 0..=19 {
        let f_name = format!("tyc2.dat.{:02}.gz", i);
        let f_path: String = format!("./data/download/tmp/{}", f_name);

        let file = File::open(f_path).unwrap();
        let mut buf = BufReader::new(file);

        let mut s = String::new();
        while buf.read_line(&mut s).unwrap() > 0 {
            let entry: Vec<&str> = s.split(|c|{c == '|'}).collect();
            //Replace above line with a FromStr trait implemented on a star quadtree entry struct?
        }
    }
}

pub fn setup_main(force_download: bool, force_extract: bool) {
    let data_path: &Path = Path::new("./data/download/I_259.tar.gz");
    let extract_path: &Path = Path::new("./data/download/extract");
    if force_download || !data_path.exists() {
        download_data();
    }
    if force_download || force_extract || !extract_path.exists() {
        extract_data();
    }
}