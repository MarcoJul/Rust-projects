use std::io::{self, Write};

pub fn entry_menu() -> usize {
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


pub fn pause() {
	print!("\nPremi INVIO per continuare...");
	io::stdout().flush().unwrap();
	let mut _input = String::new();
	io::stdin().read_line(&mut _input).unwrap();
}