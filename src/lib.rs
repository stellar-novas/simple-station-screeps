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
use screeps::Part::{Carry, Move, Work};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen;

// use rand::Rng;
// use faker_rand::en_us::names::FirstName;

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
			log::warn!(
				"[{}:{}:{}]: {:?}",
				file!(),
				line!(),
				column!(),
				&err,
			);
		}
	};
}

#[macro_export]
macro_rules! handle_info {
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
#[derive(Serialize, Deserialize, Default, Debug)]
pub enum Roles {
	Harvester,
	#[default]
	Idle,
}
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub enum Tasks {
	Harvest,
	Deliver,
	#[default]
	None,
}
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct CreepMemory  {
	pub role: Roles,
	pub task: Tasks
}

pub struct DesiredCreeps {
	
}
//
// #[derive(Serialize, Deserialize, Default, Debug)]
// #[serde(default)]
// pub struct HarvesterThoughts {
// 	my_source:
// }
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
	let raw_mem: String = screeps::raw_memory::get().into();
	trace!("memory: \n {:#?}", raw_mem);
	for creep in screeps::game::creeps().values() {
		let Some(cur_room) = creep.room() else { continue; };
		let mut creep_memory: CreepMemory = serde_wasm_bindgen::from_value(creep.memory()).unwrap();
		trace!("creep_memory: \n {:#?}", creep_memory);
		match creep_memory.role {
			Roles::Harvester => {
				if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
					creep_memory.task = Tasks::Harvest;
				}
				else if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
					creep_memory.task = Tasks::Deliver;
				}
				else if creep_memory.task == Tasks::None {
					creep_memory.task = Tasks::Harvest;
				}
				match creep_memory.task {
					Tasks::Harvest => {
						let Some(my_source) = creep.pos().find_closest_by_path(find::SOURCES_ACTIVE, None) else { continue; };
						debug!("{}'s free store capacity: {}",creep.name(), creep.store().get_free_capacity(Some(ResourceType::Energy)));
						if let Err(_e) = creep.harvest(&my_source) {
							handle_warn!(creep.move_to(my_source));
						}
					}
					Tasks::Deliver => {
						let nearest_spawn = creep.pos().find_closest_by_path(find::MY_SPAWNS, None);
						if let Some(my_spawn) = nearest_spawn {
							if my_spawn.store().get_free_capacity(Some(ResourceType::Energy)) != 0 {
								if let Err(_e) = creep.transfer(&my_spawn, ResourceType::Energy, Some(creep.store().get_capacity(Some(ResourceType::Energy)))) {
									handle_warn!(creep.move_to(my_spawn));
									continue;
								}
							}
						}
						if let Some(controller) = cur_room.controller() {
							if let Err(_e) = creep.upgrade_controller(&controller) {
								handle_warn!(creep.move_to(controller));
							}
						}

					}
					Tasks::None => {panic!("This should be unreachable!")}
				}
				creep.set_memory(&serde_wasm_bindgen::to_value(&creep_memory).unwrap())
			}
			Roles::Idle => {continue}
		}
	}
	for spawn in game::spawns().values() {
		if let Some(room) = spawn.room() {
			if room.find(find::CREEPS, None).is_empty() {
				let name = format!("Harvester-{}", game::time());
				if let Err(_e) = spawn.spawn_creep([Move, Move, Work, Carry].as_ref(), name.as_str()) {
					continue
				}
				let creep_memory = CreepMemory {role: Roles::Harvester, task: Tasks::Harvest };
				let creep = game::creeps().get(name).unwrap();
				creep.set_memory(&serde_wasm_bindgen::to_value(&creep_memory).unwrap());
			}
			
		}
	}

	info!("done! cpu: {}", game::cpu::get_used());
}


