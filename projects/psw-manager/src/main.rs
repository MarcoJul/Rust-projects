use std::io::{self, Write};

fn entry_menu() -> usize {
	println!("\n🔐 Cosa vuoi fare?");
	println!("1. Aggiungi password");
	println!("2. Visualizza password");
	println!("3. Lista passwords");
	println!("4. Elimina password");
	println!("5. Esci");
	print!("\nScegli un'opzione (1-5): ");
	io::stdout().flush().unwrap();
	
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	
	input.trim().parse().unwrap_or(0)
}
	
fn main() {
	loop {
		let selection = entry_menu();
		
		match selection {
			1 => {
				println!("Aggiungi password");
				break;
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