use std::io::{self, Write};
use crate::models::PasswordEntry;

fn prompt(label: &str) -> String {
	print!("{}", label);
	io::stdout().flush().unwrap();
	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();
	input.trim().to_string()
}

pub fn execute() {
	println!("\n Aggiungi nuova password\n");

	let name = prompt("Nome servizio (es. Gmail): ");
	let username = prompt("Username/Email: ");
	let password = prompt("Password: ");

	let entry = PasswordEntry::new(name, username, password);

	let line = entry.to_line();

	println!("\n✅ Entry creata!");
	println!("📝 Riga generata: {}", line);
	println!("🔍 Struct: {:?}", entry);
}