mod command;
mod config;
mod error;
mod random;

use self::config::Config;
use classicube_helpers::{detour::static_detour, tick::TickEventHandler, CellGetSet};
use classicube_sys::{
    BlockID, Chat_Add, Game_ChangeBlock, IGameComponent, Inventory_SelectedBlock, OwnedString,
};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    cell::{Cell, RefCell},
    os::raw::c_int,
    path::Path,
    ptr,
    sync::mpsc::channel,
    time::Duration,
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

thread_local!(
    pub static PAINT: Cell<bool> = Cell::new(false);
);

thread_local!(
    static TICK_HANDLER: RefCell<Option<TickEventHandler>> = Default::default();
);

thread_local!(
    static WATCHER: RefCell<Option<RecommendedWatcher>> = Default::default();
);

fn game_change_block_hook(x: c_int, y: c_int, z: c_int, mut block: BlockID) {
    if ENABLED.get() {
        let option_override = CONFIG.with(|cell| {
            let option = &mut *cell.borrow_mut();
            let config = option.as_ref()?;

            let target = if PAINT.get() {
                Inventory_SelectedBlock()
            } else {
                block
            };

            Some(config.choose_random_variation(x, y, z, target)?)
        });

        if let Some(block_override) = option_override {
            block = block_override;
        }
    }

    unsafe {
        DETOUR.call(x, y, z, block);
    }
}

pub fn print<S: Into<Vec<u8>>>(s: S) {
    let owned_string = OwnedString::new(s);
    unsafe {
        Chat_Add(owned_string.as_cc_string());
    }
}

extern "C" fn init() {
    command::init();

    CONFIG.with(|cell| {
        let option = &mut *cell.borrow_mut();

        match Config::load(CONFIG_FILE_PATH) {
            Ok(config) => {
                println!("loaded {:#?}", config);
                print(format!(
                    "blockref loaded {} variation groups, use \"/client BlockRef\" for help",
                    config.variation_groups.len()
                ));

                *option = Some(config);
            }

            Err(e) => {
                print(format!("blockref error: {}", e));
            }
        }
    });

    unsafe {
        DETOUR
            .initialize(Game_ChangeBlock, game_change_block_hook)
            .unwrap();
        DETOUR.enable().unwrap();
    }

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    WATCHER.with(move |cell| {
        let option = &mut *cell.borrow_mut();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(CONFIG_FILE_PATH, RecursiveMode::Recursive)
            .unwrap();

        *option = Some(watcher);
    });

    TICK_HANDLER.with(move |cell| {
        let option = &mut *cell.borrow_mut();

        let mut tick_handler = TickEventHandler::new();
        tick_handler.on(move |_| {
            for event in rx.try_iter() {
                println!("{:#?}", event);

                let maybe = match event {
                    DebouncedEvent::Create(path) => Some(path),
                    DebouncedEvent::Write(path) => Some(path),
                    DebouncedEvent::Rename(_old, path) => Some(path),

                    _ => None,
                };

                if let Some(path) = maybe {
                    if let Ok(left) = path.canonicalize() {
                        if let Ok(right) = Path::new(CONFIG_FILE_PATH).canonicalize() {
                            if left == right {
                                command::reload();
                            }
                        }
                    }
                }
            }
        });

        *option = Some(tick_handler);
    });
}

extern "C" fn free() {
    TICK_HANDLER.with(move |cell| {
        let option = &mut *cell.borrow_mut();
        drop(option.take());
    });

    WATCHER.with(move |cell| {
        let option = &mut *cell.borrow_mut();
        drop(option.take());
    });

    unsafe {
        DETOUR.disable().unwrap();
    }

    CONFIG.with(|cell| {
        let option = &mut *cell.borrow_mut();
        drop(option.take());
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
    OnNewMapLoaded: None,
    // Next component in linked list of components.
    next: ptr::null_mut(),
};
