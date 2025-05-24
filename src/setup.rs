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
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{fs, io::Write};
use std::fs::{canonicalize, File, OpenOptions};
use std::path::{Path, PathBuf};

use curl::easy::{Easy, WriteError};
use spherical_quadtree::{SphQtRoot, StarData};

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
fn generate_cpu_quadtree_thread(sph_qt: Arc<Mutex<&mut SphQtRoot>>, file_index: i32) {
    //Launch a thread for each star data file and one more to add the stars to the quadtree with an mpsc model

    let f_name = format!("tyc2.dat.{:02}", file_index);
    let f_path: String = format!("./data/download/extract/{}", f_name);

    println!("Attempting to open{}", &f_path);

    let file = File::open(f_path).unwrap();
    let mut buf = BufReader::new(file);

    let mut s = String::new();
    while buf.read_line(&mut s).unwrap() > 0 {
        let entry: Vec<&str> = s.split(|c|{c == '|'}).map(|x| {x.trim()}).collect();
        //println!("{:?}", entry);
        if entry[19].parse::<f32>().expect("VT not a valid f32") > 10.0 || entry[1].contains(|x|{x=='P' || x=='X'}) {
            continue;
        }
        else {
            let star: spherical_quadtree::StarData = StarData {
                ra: entry[2].parse().expect("RA not a valid f32"),
                dec: entry[3].parse().expect("Dec not a valid f32"),
                bt: entry[17].parse().expect("BT not a valid f32"),
                vt: entry[19].parse().expect("VT not a valid f32")
            };
            let mut sph_qt_locked = sph_qt.lock().unwrap(); //Lock the mutex
            sph_qt_locked.add(star);
            drop(sph_qt_locked); //Unlock the mutex
            //Replace above line with a FromStr trait implemented on a star quadtree entry struct?
        }
    }
}

fn generate_cpu_quadtree(sph_qt: &mut spherical_quadtree::SphQtRoot) {
    let qt_mutex = Arc::new(Mutex::new(sph_qt));

    let start = Instant::now();
    thread::scope(|s| {
        let mut handles = Vec::new();
        for i in 0..=19 {
            let qt_mutex = Arc::clone(&qt_mutex);
            handles.push(s.spawn(move || generate_cpu_quadtree_thread(qt_mutex, i)));
        }
        for h in handles {
            h.join().unwrap();
        }
    });
    let duration = start.elapsed();
    println!("Quadtree construction took {} seconds", duration.as_secs_f32());
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

    let mut quadtree = SphQtRoot::new();
    generate_cpu_quadtree(&mut quadtree);
}