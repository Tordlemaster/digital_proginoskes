//Download the Tycho-2 data from https://cdsarc.u-strasbg.fr/viz-bin/Cat?I/259#/browse
//Unzip it by calling tar with std::process
//process the data culling out stars below a specified magnitude
//and then create a new file with either the quadtree format or the GPU instanced points format
//(icosahedron spherical quadtree or cube spherical quadtree) (cube might be easier)
//write it with serde???

//Attempt to open and read from data file and if fail then download it

//Each cell of the quadtree should have an average color value so that
//if there are many stars packed into 1 pixel the quadtree navigation can stop at that point and just return the precalculated value

use std::io::{BufRead, BufReader, BufWriter, Read};
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write};
use std::fs::{canonicalize, File, OpenOptions};
use std::path::{Path};

use curl::easy::{Easy, WriteError};
use crate::spherical_quadtree::{self, SphQtRoot, StarData};

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

//Read through the star data and write the data from stars brighter than min_magnitude to a file.
fn prune_stars(min_magnitude: f32) {

    fs::create_dir_all("./data/cache").unwrap();
    let file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("./data/cache/pruned_stars.dat")
        .unwrap();
    let mut write_buf = BufWriter::new(file);
    
    let mut star_count: u64 = 0;
    let mut pruned_star_count: u64 = 0;

    let start = Instant::now();
    
    for i in 0..=19 {
        let f_name = format!("tyc2.dat.{:02}", i);
        let f_path: String = format!("./data/download/extract/{}", f_name);

        //println!("Attempting to open {}", &f_path);

        let file = File::open(f_path).unwrap();
        let mut buf = BufReader::new(file);

        let mut s = String::new();
        while buf.read_line(&mut s).unwrap() > 0 {
            //TODO NOT CORRECTLY READING UNTIL EOL

            let entry: Vec<&str> = s.split(|c|{c == '|'}).map(|x| {x.trim()}).collect();
            /*println!("{:?}", entry);
            if (entry.len() < 19) {
                panic!("entry too short");
            }*/

            if entry[2].len() > 0 && entry[3].len() > 0 && entry[17].len() > 0 && entry[19].len() > 0 && entry[19].parse::<f32>().unwrap() < min_magnitude && !entry[1].contains(|x|{x=='P' || x=='X'}) {
                let ra = entry[2].parse::<f32>().expect("RA not a valid f32").to_ne_bytes();
                let dec = entry[3].parse::<f32>().expect("Dec not a valid f32").to_ne_bytes();
                let bt = entry[17].parse::<f32>().expect("BT not a valid f32").to_ne_bytes();
                let vt = entry[19].parse::<f32>().expect("VT not a valid f32").to_ne_bytes();
                match write_buf.write(&ra) {
                    Ok(n) => Ok(n),
                    Err(_) => Err(WriteError::Pause)
                }.expect("Failed to write");
                match write_buf.write(&dec) {
                    Ok(n) => Ok(n),
                    Err(_) => Err(WriteError::Pause)
                }.expect("Failed to write");
                match write_buf.write(&bt) {
                    Ok(n) => Ok(n),
                    Err(_) => Err(WriteError::Pause)
                }.expect("Failed to write");
                match write_buf.write(&vt) {
                    Ok(n) => Ok(n),
                    Err(_) => Err(WriteError::Pause)
                }.expect("Failed to write");

                pruned_star_count += 1;
                
            }
            s.clear();
            
            star_count += 1;
            if start.elapsed().as_secs_f32().fract()<0.05 {
                print!("\r{} stars processed", star_count);
            }
        }
    }
    let duration = start.elapsed();
    println!("\nPruned list of {} stars into {} entries in {} seconds", star_count, pruned_star_count, duration.as_secs_f32());
}

//Have one thread that processes the file and appends the data to a structure and another that constructs the quadtree
fn generate_cpu_quadtree(sph_qt: &mut spherical_quadtree::SphQtRoot) {
    let file = File::open("./data/cache/pruned_stars.dat").unwrap();
    let mut buf = BufReader::new(file);
    let mut star_entry: [u8; 16] = [0; 16];

    let mut star_count: u64 = 0;

    let start = Instant::now();

    while buf.read_exact(&mut star_entry).is_ok() {
        let star: spherical_quadtree::StarData = StarData {
            ra: f32::from_ne_bytes(star_entry[0..4].try_into().unwrap()),
            dec: f32::from_ne_bytes(star_entry[4..8].try_into().unwrap()),
            bt: f32::from_ne_bytes(star_entry[8..12].try_into().unwrap()),
            vt: f32::from_ne_bytes(star_entry[12..16].try_into().unwrap())
        };
        sph_qt.add(star);
        star_count += 1;
        if start.elapsed().as_secs_f32().fract()<0.05 {
            print!("\r{} stars processed", star_count);
        }
    }

    let duration = start.elapsed();
    println!("\nCreated quadtree from {} stars in {} seconds", star_count, duration.as_secs_f32());
}

pub fn setup_main(force_download: bool, force_extract: bool, force_prune: bool, quadtree: &mut spherical_quadtree::SphQtRoot) {
    let data_path: &Path = Path::new("./data/download/I_259.tar.gz");
    let extract_path: &Path = Path::new("./data/download/extract");
    let cache_path: &Path = Path::new("./data/cache/pruned_stars.dat");

    if force_download || !data_path.exists() {
        download_data();
    }
    if force_download || force_extract || !extract_path.exists() {
        extract_data();
    }
    if force_prune || force_download || force_extract || !cache_path.exists() {
        prune_stars(6.0);
    }

    //let mut quadtree = SphQtRoot::new();
    generate_cpu_quadtree(quadtree);
}