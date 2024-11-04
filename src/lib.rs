use std::{
	cell::RefCell,
	collections::{hash_map::Entry, HashMap, HashSet},
};
use std::fmt::{Display, Formatter};
use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::{
	constants::{ErrorCode, Part, ResourceType},
	enums::StructureObject,
	find, game,
	local::ObjectId,
	objects::{Creep, Source, StructureController},
	prelude::*,
};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

mod logging;

#[macro_export]
macro_rules! handle_err {
	($e:expr) => {
		if let Err(err) = $e {
			log::error!(
				"[{}:{}:{}]: {:?}\n\tsrc = {}",
				file!(),
				line!(),
				column!(),
				&err,
				{
					let src = stringify!($e);
					if src.len() > 45 {
						format!("{}...", &src[..40])
					} else {
						src.to_string()
					}
				}
			);
		}
	};
}

#[macro_export]
macro_rules! handle_warn {
	($e:expr) => {
		if let Err(err) = $e {
			log::info!(
				"[{}:{}:{}]: {:?}",
				file!(),
				line!(),
				column!(),
				&err,
			);
		}
	};
}

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = "loop")]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Debug);
	});

	debug!("loop starting! CPU: {}", game::cpu::get_used());
	// TODO: Try to harvest and move if it errors.
	// Also handle moving back to spawn if store (inventory) is full.
	for creep in screeps::game::creeps().values() {
		// let Some(cur_room) = creep.room() else { continue; };
		// if creep.store().get_free_capacity(Some(ResourceType::Energy)) != 0 {
		// 	let active_sources = cur_room.find(find::SOURCES_ACTIVE, None);
		// 	debug!("creep active sources: {:#?}", active_sources);
		// 	let Some(dest) = creep.pos().find_closest_by_path(find::SOURCES_ACTIVE, None) else { continue; };
		// 	debug!("creep dest: {:#?}", dest);
		// 	handle_err!(creep.move_to(dest));
		// 	handle_warn!(creep.harvest(&creep.pos().find_closest_by_range(find::SOURCES_ACTIVE).unwrap()));
		// }
		// else {
		// 	handle_err!(creep.move_to(creep.pos().find_closest_by_path(find::MY_SPAWNS, None).unwrap()));
		// }
		
        let Some(cur_room) = creep.room() else { continue; };
		let memory = creep.memory();
		debug!("creep memory: {:#?}", memory);
		
	}

	info!("done! cpu: {}", game::cpu::get_used());
}


