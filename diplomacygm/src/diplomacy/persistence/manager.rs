use std::{collections::HashMap, sync::{Mutex, RwLock}};

use serenity::all::GuildId;
use tokio::task;

use crate::diplomacy::map_parser::vector::vector::Parser;

use super::{board::Board, phase::Phase};

//  Manager acts as an intermediary between Bot (the Discord API), Board (the board state), the database.
pub struct Manager {
    // database: ,
    _boards: RwLock<HashMap<GuildId, Mutex<Board>>>,
}

impl Manager {
    pub fn new() -> Manager {
        //         self._database = database.get_connection()
        //         self._boards: dict[int, Board] = self._database.get_boards()
        //         # TODO: have multiple for each variant?
        //         # do it like this so that the parser can cache data between board initilizations
        Manager {
            _boards: RwLock::new(HashMap::new()),
        }
    }

    pub fn list_servers(&self) -> Vec<GuildId> {
        let boards = self._boards.read().unwrap();

        boards.keys().map(|x| { x.clone() }).collect::<Vec<GuildId>>()
    }

    pub async fn create_game(&self, server_id: &GuildId, gametype: String) -> String {
        {
            let boards = self._boards.read().unwrap();
            if boards.contains_key(server_id) {
                panic!("todo actually do anyhow")
    //             raise RuntimeError("A game already exists in this server.")
            }
        }

        // TODO logger.info(f"Creating new game in server {server_id}")s

        let result = task::spawn_blocking(|| Parser::new(gametype)).await.unwrap();

        let mut boards = self._boards.write().unwrap();
        let mut new_board = Board::new(
            Vec::new(),
            Vec::new(),
            // HashSet::new(),
            Phase::SpringMoves, 
            String::from("none")            
        );

        new_board.board_id = Some(server_id.clone());
//         self._database.save_board(server_id, self._boards[server_id])
        boards.insert(server_id.clone(), Mutex::new(new_board));
        println!("{}", server_id.clone());
        

        // self._boards[server_id] = get_parser(gametype).parse()

//         return f"{self._boards[server_id].data['name']} game created"
        return format!("{} game created", "idk");

    }

    pub fn change_fish(&self, server_id: GuildId, amount: i64) -> i64 {
        let boards = self._boards.read().unwrap();
        let mut board = boards.get(&server_id).unwrap_or_else(|| todo!()).lock().unwrap();
        board.fish += amount;

        board.fish
    }
}



// import logging
// import time

// from diplomacy.adjudicator.adjudicator import make_adjudicator
// from diplomacy.adjudicator.mapper import Mapper
// from diplomacy.map_parser.vector.vector import get_parser
// from diplomacy.persistence import phase
// from diplomacy.persistence.board import Board
// from diplomacy.persistence.db import database
// from diplomacy.persistence.player import Player

// logger = logging.getLogger(__name__)


// class Manager:
//     """Manager acts as an intermediary between Bot (the Discord API), Board (the board state), the database."""

//     def total_delete(self, server_id: int):
//         self._database.total_delete(self._boards[server_id])
//         del self._boards[server_id]

//     def draw_moves_map(self, server_id: int, player_restriction: Player | None) -> str:
//         start = time.time()

//         svg = Mapper(self._boards[server_id]).draw_moves_map(self._boards[server_id].phase, player_restriction)

//         elapsed = time.time() - start
//         logger.info(f"manager.draw_moves_map.{server_id}.{elapsed}s")
//         return svg

//     def adjudicate(self, server_id: int) -> str:
//         start = time.time()

//         # mapper = Mapper(self._boards[server_id])
//         # mapper.draw_moves_map(None)
//         adjudicator = make_adjudicator(self._boards[server_id])
//         # TODO - use adjudicator.orders() (tells you which ones succeeded and failed) to draw a better moves map
//         new_board = adjudicator.run()
//         new_board.phase = new_board.phase.next
//         if new_board.phase.name == "Spring Moves":
//             new_board.year += 1
//         logger.info("Adjudicator ran successfully")
//         self._boards[server_id] = new_board
//         self._database.save_board(server_id, new_board)
//         mapper = Mapper(new_board)
//         svg = mapper.draw_current_map()

//         elapsed = time.time() - start
//         logger.info(f"manager.adjudicate.{server_id}.{elapsed}s")
//         return svg

//     def rollback(self, server_id: int) -> tuple[str, str]:
//         logger.info(f"Rolling back in server {server_id}")
//         board = self._boards[server_id]
//         # TODO: what happens if we're on the first phase?
//         last_phase = board.phase.previous
//         last_phase_year = board.year
//         if board.phase.name == "Spring Moves":
//             last_phase_year -= 1

//         old_board = self._database.get_board(board.board_id, last_phase, last_phase_year, board.fish)
//         if old_board is None:
//             raise ValueError(f"There is no {last_phase_year} {last_phase.name} board for this server")

//         self._database.delete_board(board)
//         self._boards[server_id] = old_board
//         mapper = Mapper(old_board)
//         return f"Rolled back to {old_board.get_phase_and_year_string()}", mapper.draw_current_map()

//     def reload(self, server_id: int) -> tuple[str, str]:
//         logger.info(f"Reloading server {server_id}")
//         board = self._boards[server_id]

//         loaded_board = self._database.get_board(server_id, board.phase, board.year, board.fish)
//         if loaded_board is None:
//             raise ValueError(f"There is no {board.year} {board.phase.name} board for this server")

//         self._boards[server_id] = loaded_board
//         mapper = Mapper(loaded_board)
//         return f"Reloaded board for phase {loaded_board.get_phase_and_year_string()}", mapper.draw_current_map()
