use crate::{
    error::*,
    print,
    random::{get_rng, Seed},
};
use classicube_sys::{BlockID, World};
use rand::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, prelude::*, BufReader, Write},
    os::raw::c_int,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Config {
    // weighted variations
    pub variation_groups: HashMap<BlockID, Vec<(BlockID, usize)>>,
    pub path: PathBuf,
}

impl Config {
    pub fn reload(&mut self) -> Result<()> {
        self.variation_groups = Self::load(&self.path)?.variation_groups;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        match File::open(&path) {
            Ok(file) => Ok(Self {
                variation_groups: Self::read_from_file(file)?,
                path: path.to_path_buf(),
            }),

            Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        // create blank file with comment
                        {
                            let mut f = File::create(&path)?;
                            writeln!(f, "# [target] [1] [2] [3]...")?;
                        }

                        print(format!("created new file {:?}", path));

                        Ok(Self {
                            variation_groups: Default::default(),
                            path: path.to_path_buf(),
                        })
                    }

                    _ => Err(e.into()),
                }
            }
        }
    }

    fn read_from_file(file: File) -> Result<HashMap<BlockID, Vec<(BlockID, usize)>>> {
        let reader = BufReader::new(file);

        let mut variation_groups = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let mut args = line.split_whitespace();
            let target: BlockID = args.next().chain_err(|| "missing target")?.parse()?;

            let mut weighted_variations = Vec::new();
            for arg in args {
                let mut split = arg.split(':');
                let block = split.next().chain_err(|| "wtf")?.parse()?;
                let weight = split.next().unwrap_or("1").parse()?;
                weighted_variations.push((block, weight));
            }
            variation_groups.insert(target, weighted_variations);
        }

        Ok(variation_groups)
    }

    pub fn choose_random_variation(
        &self,
        x: c_int,
        y: c_int,
        z: c_int,
        block: BlockID,
    ) -> Option<BlockID> {
        let weighted_variations = self.variation_groups.get(&block)?;

        let mut rng = get_rng(&Seed {
            x,
            y,
            z,
            volume: unsafe { World.Volume },
        });

        let block = weighted_variations
            .choose_weighted(&mut rng, |weighted| weighted.1)
            .map(|a| a.0)
            .ok()?;

        Some(block)
    }
}
