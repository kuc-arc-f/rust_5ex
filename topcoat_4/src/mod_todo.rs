use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Todo {
    id: u32,
    title: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoData {
    items: Vec<Todo>,
    max_id: u32,
}

const DATA_FILE: &str = "todos.json";

fn load_data() -> TodoData {
    if let Ok(content) = fs::read_to_string(DATA_FILE) {
        serde_json::from_str(&content).unwrap_or_else(|_| TodoData {
            items: Vec::new(),
            max_id: 0,
        })
    } else {
        TodoData {
            items: Vec::new(),
            max_id: 0,
        }
    }
}

fn save_data(data: &TodoData) {
    let json = serde_json::to_string_pretty(data).unwrap();
    fs::write(DATA_FILE, json).unwrap();
}

pub fn add_todo(title: &str) {
    let mut data = load_data();
    data.max_id += 1;
    data.items.push(Todo {
        id: data.max_id,
        title: title.to_string(),
    });
    save_data(&data);
    println!("Added todo #{}: {}", data.max_id, title);
}

pub fn list_todos() {
    let data = load_data();
    if data.items.is_empty() {
        println!("No todos found.");
        return;
    }
    println!("Your todos:");
    for todo in data.items {
        println!("  #{}: {}", todo.id, todo.title);
    }
}

pub fn list_todo_json() -> std::result::Result<String, String> {
    let data = load_data();
    if data.items.is_empty() {
        println!("No todos found.");
        return Err("No todos found.".to_string());
    }
    println!("Your todos:");
    let todo_items = data.items;
    for todo in &todo_items {
        println!("  #{}: {}", todo.id, todo.title);
    }
    let out = &todo_items.clone();
    let j1 = json!(&out);
    Ok(j1.to_string())
}

pub fn delete_todo(id: u32) {
    let mut data = load_data();
    let before_len = data.items.len();
    data.items.retain(|todo| todo.id != id);
    if data.items.len() == before_len {
        println!("Todo #{} not found.", id);
        return;
    }
    save_data(&data);
    println!("Deleted todo #{}", id);
}

fn clear_todos() {
    print!("Are you sure you want to delete all todos? (y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "y" {
        let mut data = load_data();
        data.items.clear();
        data.max_id = 0;
        save_data(&data);
        println!("All todos cleared.");
    } else {
        println!("Operation cancelled.");
    }
}
