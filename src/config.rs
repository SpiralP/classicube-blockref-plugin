use crate::{
    error::*,
    random::{get_rng, Seed},
};
use classicube_sys::{BlockID, World};
use rand::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter, Write},
    os::raw::c_int,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Config {
    pub variations: HashMap<BlockID, Vec<BlockID>>,
    pub path: PathBuf,
}

impl Config {
    pub fn reload(&mut self) -> Result<()> {
        self.variations = Self::load(&self.path)?.variations;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        match File::open(&path) {
            Ok(file) => Ok(Self {
                variations: Self::read_from_file(file)?,
                path: path.as_ref().to_path_buf(),
            }),

            Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        // create blank file with comment

                        let mut f = File::create(&path)?;
                        writeln!(f, "# [target] [1] [2] [3]...")?;

                        Ok(Self {
                            variations: Default::default(),
                            path: path.as_ref().to_path_buf(),
                        })
                    }

                    _ => Err(e.into()),
                }
            }
        }
    }

    fn read_from_file(file: File) -> Result<HashMap<BlockID, Vec<BlockID>>> {
        let reader = BufReader::new(file);

        let mut variations = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let mut args: Vec<BlockID> = line
                .split_whitespace()
                .map(|s| s.parse())
                .collect::<std::result::Result<_, _>>()?;

            let target: BlockID = *args.get(0).chain_err(|| "missing target")?;
            let vars: Vec<BlockID> = args.drain(..).skip(1).collect();

            variations.insert(target, vars);
        }

        Ok(variations)
    }

    fn save_to_file(file: File) -> Result<()> {
        let writer = BufWriter::new(file);
        //

        Ok(())
    }

    pub fn choose_random_variation(
        &self,
        x: c_int,
        y: c_int,
        z: c_int,
        block: BlockID,
    ) -> Option<BlockID> {
        let variations = self.variations.get(&block)?;

        Some(*variations.choose(&mut get_rng(&Seed {
            x,
            y,
            z,
            volume: unsafe { World.Volume },
        }))?)
    }
}
