use std::{
	// cell::RefCell,
	collections::{hash_map::Entry, HashMap, HashSet},
};
use std::fmt::{Display, Formatter};
use fake::Fake;
use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::{constants::{ErrorCode, Part, ResourceType}, enums::StructureObject, find, game, local::ObjectId, objects::{Creep, Source, StructureController}, prelude::*, Room, StructureSpawn};
use screeps::Part::{Carry, Move, RangedAttack, Work};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use fake::faker::name::raw::*;
use fake::locales::*;

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
	Fighter,
	#[default]
	Idle,
}
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub enum Tasks {
	Harvest,
	Deliver,
	Patrol,
	#[default]
	None,
}
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct CreepMemory  {
	pub role: Roles,
	pub task: Tasks
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct RoomMemory {
	pub wanted_creeps: CreepCounts,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct CreepCounts {
	harvester: u8,
	fighter: u8
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

#[wasm_bindgen]
pub fn set_log_level(level: String) -> Result<(), String>{
	let log_level = match level.as_str() {
		"error" => LevelFilter::Error,
		"warn" => LevelFilter::Warn,
		"info" => LevelFilter::Info,
		"debug" => LevelFilter::Debug,
		"trace" => LevelFilter::Trace,
		_ => return Err("Invalid log level".to_string())
	};
	log::set_max_level(log_level);
	Ok(())
}

pub fn get_room(room_id: String) -> Result<Room, String> {
	let room = game::rooms().get(room_id.parse().unwrap());
	if let Some(room) = room {
		return Ok(room);
	};
	Err("Room not found".to_string())
}

#[wasm_bindgen]
pub fn dump_room_memory(room_id: String) -> Result<String, String> {
	let room = get_room(room_id).or_else(|e| Err(e))?;
	let room_memory: RoomMemory = serde_wasm_bindgen::from_value(room.memory()).unwrap();
	return Ok(format!("room_memory: \n {:#?}", room_memory));
}

#[wasm_bindgen]
pub fn dump_creep_memory(creep_name: String) -> Result<String, String> {
	let creep = game::creeps().get(creep_name);
	if let Some(creep) = creep {
		let creep_memory: CreepMemory = serde_wasm_bindgen::from_value(creep.memory()).unwrap();
		return Ok(format!("creep_memory: \n {:#?}", creep_memory));
	};
	Err("Creep not found".to_string())
}

pub fn spawn_creep(spawn: &StructureSpawn, role: Roles) -> Result<(), ErrorCode> {
	let name: String = Name(EN).fake();
	let body: Vec<Part> = match role {
		Roles::Harvester => vec![Work, Move, Carry, Move, Carry],
		Roles::Fighter => vec![Move, Move, Move, RangedAttack],
		_ => panic!("Invalid role")
	};
	match role {
		Roles::Harvester => {
			let body: Vec<Part> = vec![Work, Move, Carry, Move, Carry];
			let creep_memory = CreepMemory {role: Roles::Harvester, task: Tasks::Harvest};
		}
		Roles::Fighter => {
			let body: Vec<Part> = vec![Move, Move, Move, RangedAttack];
			let creep_memory = CreepMemory {role: Roles::Fighter, task: Tasks::Patrol};
		}
		Roles::Idle => {
			let body: Vec<Part> = vec![Work, Move, Carry, Move, Carry];
			let creep_memory = CreepMemory {role: Roles::Idle, task: Tasks::None};
		}
	}
	if let Err(_e) = spawn.spawn_creep(body.as_ref(), name.as_str()) {
		return Err(ErrorCode::NotEnough);
	}
	Ok(())
}

#[wasm_bindgen]
pub fn get_creeps(roomid: String) -> Result<String, String> {
	let room = get_room(roomid)?;
	let room_memory: RoomMemory = serde_wasm_bindgen::from_value(room.memory()).unwrap();
	Ok(format!("Creeps: \n {:#?}", room_memory.wanted_creeps))
}

#[wasm_bindgen]
pub fn set_creeps(roomid: String, role: String, count: u8) -> Result<(), String> {
	let room = get_room(roomid)?;
	let mut room_memory: RoomMemory = serde_wasm_bindgen::from_value(room.memory()).unwrap();
	match role.as_str() {
		"harvester" => {room_memory.wanted_creeps.harvester = count}
		"fighter" => {room_memory.wanted_creeps.fighter = count}
		_ => {return Err("Invalid role".to_string())}
	}
	room.set_memory(&serde_wasm_bindgen::to_value(&room_memory).unwrap());
	Ok(())
}

#[wasm_bindgen(js_name = "loop")]
pub fn game_loop() {
	INIT_LOGGING.call_once(|| {
		// show all output of Info level, adjust as needed
		logging::setup_logging(logging::Trace);
	});
	debug!("loop starting! CPU: {}", game::cpu::get_used());
	let raw_mem: String = screeps::raw_memory::get().into();
	trace!("memory: \n {:#?}", raw_mem);
	for creep in game::creeps().values() {
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
						else if let Some(controller) = cur_room.controller() {
							if let Err(_e) = creep.upgrade_controller(&controller) {
								handle_warn!(creep.move_to(controller));
							}
						}
					}
					Tasks::Patrol => {panic!("Impossible task for harvester")}
					Tasks::None => {creep_memory.task = Tasks::Harvest;}
				}
			}
			_ => {continue}
		}
		creep.set_memory(&serde_wasm_bindgen::to_value(&creep_memory).unwrap())
	}
	for spawn in game::spawns().values() {
		if let Some(room) = spawn.room() {
			let room_memory: RoomMemory = serde_wasm_bindgen::from_value(room.memory()).unwrap();
			trace!("room_memory: \n {:#?}", room_memory);
			let mut current_creeps = CreepCounts {harvester: 0, fighter: 0};
			for creep in room.find(find::CREEPS, None) {
				let mut creep_memory: CreepMemory = serde_wasm_bindgen::from_value(creep.memory()).unwrap();
				match creep_memory.role {
					Roles::Harvester => {current_creeps.harvester += 1}
					Roles::Fighter => {current_creeps.fighter += 1}
					_ => {creep_memory.role = Roles::Idle}
				}
			}
			trace!("current_creeps: \n {:#?}", current_creeps);
			if current_creeps.harvester < room_memory.wanted_creeps.harvester {
				handle_info!(spawn_creep(&spawn, Roles::Harvester));
			}
			else if current_creeps.fighter < room_memory.wanted_creeps.fighter {
				handle_info!(spawn_creep(&spawn, Roles::Fighter));
			}
		}
	}

	info!("done! cpu: {}", game::cpu::get_used());
}


