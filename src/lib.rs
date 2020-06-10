use classicube_helpers::detour::static_detour;
use classicube_sys::{cc_bool, BlockID, Chat_Send, Game_ChangeBlock, IGameComponent, OwnedString};
use std::{os::raw::c_int, ptr};

// toggle
// files to store variations
// &rEmpy: &fso i suppose on a line youd have [target][1][2][3] etc
// put comment at top of file

// fn calculate_hash<T: Hash>(t: &T) -> u64 {
//     let mut s = DefaultHasher::new();
//     t.hash(&mut s);
//     s.finish()
//   }

//   fn random() {

//   let hash = calculate_hash(&HashedInfo {
//     real_name,
//     messages_said,
//   });

//   Box::new(ChaChaRng::seed_from_u64(hash))
// }

static_detour! {
    static DETOUR: unsafe extern "C" fn(c_int, c_int, c_int, BlockID);
}

fn game_change_block_hook(x: c_int, y: c_int, z: c_int, block: BlockID) {
    println!("Game_ChangeBlock {:?} {:?} {:?} {:?}", x, y, z, block);

    unsafe {
        DETOUR.call(x, y, z, block);
    }
}

extern "C" fn init() {
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
