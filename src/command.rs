use crate::{error::*, print, CONFIG, ENABLED};
use classicube_helpers::CellGetSet;
use classicube_sys::OwnedChatCommand;
use std::{cell::Cell, os::raw::c_int, slice};

pub fn reload() {
    let result = CONFIG.with(|cell| {
        let option = &mut *cell.borrow_mut();
        let config = option.as_mut().chain_err(|| "no config loaded??")?;
        config.reload()?;

        print(format!(
            "blockref reloaded {} targets",
            config.variations.len()
        ));

        Ok::<_, Error>(())
    });

    if let Err(e) = result {
        print(format!("reload failed: {}", e));
    }
}

extern "C" fn c_chat_command_callback(args: *const classicube_sys::String, args_count: c_int) {
    let args = unsafe { slice::from_raw_parts(args, args_count as _) };
    let args: Vec<String> = args.iter().map(|cc_string| cc_string.to_string()).collect();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match args.as_slice() {
        ["reload"] => {
            reload();
        }

        ["disable"] => {
            ENABLED.set(false);
            print("BlockRef disabled");
        }

        ["enable"] => {
            ENABLED.set(true);
            print("BlockRef enabled");
        }

        _ => {
            print("/client BlockRef reload");
            print("/client BlockRef disable");
            print("/client BlockRef enable");
        }
    }
}

thread_local!(
    static COMMAND: Cell<Option<OwnedChatCommand>> = Default::default();
);

pub fn init() {
    COMMAND.with(|cell| {
        let mut command = OwnedChatCommand::new(
            "BlockRef",
            c_chat_command_callback,
            false,
            vec!["/client BlockRef"],
        );

        command.register();

        cell.set(Some(command));
    });
}

pub fn free() {
    COMMAND.with(|cell| {
        drop(cell.replace(None));
    });
}
