mod command;
mod config;
mod error;
mod random;

use self::config::Config;
use classicube_helpers::{detour::static_detour, CellGetSet};
use classicube_sys::{BlockID, Chat_Add, Game_ChangeBlock, IGameComponent, OwnedString};
use std::{
    cell::{Cell, RefCell},
    os::raw::c_int,
    ptr,
};

const CONFIG_FILE_PATH: &str = "plugins/blockref_variations.txt";

// toggle
// files to store variations
// &rEmpy: &fso i suppose on a line youd have [target][1][2][3] etc
// put comment at top of file

static_detour! {
    static DETOUR: unsafe extern "C" fn(c_int, c_int, c_int, BlockID);
}

thread_local!(
    pub static CONFIG: RefCell<Option<Config>> = Default::default();
);

thread_local!(
    pub static ENABLED: Cell<bool> = Cell::new(false);
);

fn game_change_block_hook(x: c_int, y: c_int, z: c_int, mut block: BlockID) {
    if ENABLED.get() {
        let option_override = CONFIG.with(|cell| {
            let option = &mut *cell.borrow_mut();
            let config = option.as_ref()?;

            Some(config.choose_random_variation(x, y, z, block)?)
        });

        if let Some(block_override) = option_override {
            block = block_override;
        }
    }

    unsafe {
        DETOUR.call(x, y, z, block);
    }
}

fn load_config(notify: bool) {
    CONFIG.with(|cell| {
        let option = &mut *cell.borrow_mut();

        match Config::load(CONFIG_FILE_PATH) {
            Ok(config) => {
                // println!("loaded {:#?}", config);
                if notify {
                    print(format!(
                        "blockref loaded {} targets, use \"/client BlockRef\" for help",
                        config.variations.len()
                    ));
                }

                *option = Some(config);
            }

            Err(e) => {
                print(format!("blockref error: {}", e));
            }
        }
    });
}

extern "C" fn on_new_map_loaded() {
    load_config(false);
}

pub fn print<S: Into<Vec<u8>>>(s: S) {
    let owned_string = OwnedString::new(s);
    unsafe {
        Chat_Add(owned_string.as_cc_string());
    }
}

extern "C" fn init() {
    command::init();

    load_config(true);

    unsafe {
        DETOUR
            .initialize(Game_ChangeBlock, game_change_block_hook)
            .unwrap();
        DETOUR.enable().unwrap();
    }
}

extern "C" fn free() {
    unsafe {
        DETOUR.disable().unwrap();
    }

    CONFIG.with(|cell| {
        let option = &mut *cell.borrow_mut();
        drop(option.take().unwrap());
    });

    command::free();
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
    // Called when the game is being loaded.
    Init: Some(init),
    // Called when the component is being freed. (e.g. due to game being closed)
    Free: Some(free),
    // Called to reset the component's state. (e.g. reconnecting to server)
    Reset: None,
    // Called to update the component's state when the user begins loading a new map.
    OnNewMap: None,
    // Called to update the component's state when the user has finished loading a new map.
    OnNewMapLoaded: Some(on_new_map_loaded),
    // Next component in linked list of components.
    next: ptr::null_mut(),
};
