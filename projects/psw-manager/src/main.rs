use std::io::{self, Write};
mod models;
mod commands;
mod utils;
	
fn main() {
	loop {
		let selection = utils::entry::entry_menu();
		
		match selection {
			1 => {
				println!("Aggiungi password");
				commands::add::execute();
				utils::pause();
			}
			2 => {
				println!("Visualizza password");
				break;
			}
			3 => {
				println!("Lista passwords");
				break;
			}
			4 => {
				println!("Elimina password");
				break;
			}
			5 => {
				println!("Arrivederci!");
				break;
			}
			_ => println!("Opzione non valida!"),
		}
	}
}
