#[derive(Debug, Clone)]
pub struct PasswordEntry {
		pub id: Option<u64>,
		pub name: String,
		pub username: String,
		pub password: String,
}

impl PasswordEntry {
	pub fn new(name: String, username: String, password: String) -> Self {
		PasswordEntry {
			id: None,
			name, 
			username,
			password
		}
	}

	pub fn to_line(&self) -> String {
		let id_str = match self.id {
			Some(id) => id.to_string(),
			None => String::from("")
		};

		format!(
			"{}|{}|{}|{}",
			id_str,
			self.name,
			self.username,
			self.password
		)
	}
}