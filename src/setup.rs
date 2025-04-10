//Download the Tycho-2 data from https://cdsarc.u-strasbg.fr/viz-bin/Cat?I/259#/browse
//Unzip it by calling tar with std::process
//process the data culling out stars below a specified magnitude
//and then create a new file with either the quadtree format or the GPU instanced points format
//write it with serde???

//Attempt to open and read from data file and if fail then download it

//Each cell of the quadtree should have an average color value so that
//if there are many stars packed into 1 pixel the quadtree navigation can stop at that point and just return the precalculated value

use std::{fs, io::Write};
use std::fs::OpenOptions;
use std::path::Path;

use curl::easy::{Easy, WriteError};

const TYCHO_2_URL: &str = "https://cdsarc.u-strasbg.fr/viz-bin/nph-Cat/tar.gz?I/259";

fn data_write_fn(data: &[u8]) -> Result<usize, WriteError> {
    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("./data/download/I_259.tar.gz")
        .unwrap();
    let r = match file.write_all(data) {
        Ok(()) => Ok(data.len()),
        Err(_e) => Err(WriteError::Pause)
    };
    r
}

fn download_data() {
    fs::create_dir_all("./data/download").unwrap();

    let mut easy = Easy::new();
    easy.url(TYCHO_2_URL).unwrap();
    easy.write_function(data_write_fn).unwrap();
    easy.perform().unwrap();
}

pub fn setup_main(force_download: bool) {
    let data_path: &Path = Path::new("./data/download");
    if !data_path.exists() | force_download {
        download_data();
    }
}