use std::io::{self, Write};
use crate::models::PasswordEntry;

pub fn execute() {
	println!("\n Aggiungi nuova password\n");

	print!("Nome servizio (es. Gmail): ");
	io::stdout().flush().unwrap();
	let mut name = String::new();
	io::stdin().read_line(&mut name).unwrap();
	let name = name.trim().to_string();

	print!("Username/Email: ");
	io::stdout().flush().unwrap();
	let mut username = String::new();
	io::stdin().read_line(&mut username).unwrap();
	let username = username.trim().to_string();

	print!("Password: ");
	io::stdout().flush().unwrap();
	let mut password = String::new();
	io::stdin().read_line(&mut password).unwrap();
	let password = password.trim().to_string();

	let entry = PasswordEntry::new(name, username, password);

	let line = entry.to_line();

	println!("\n✅ Entry creata!");
	println!("📝 Riga generata: {}", line);
	println!("🔍 Struct: {:?}", entry);
}