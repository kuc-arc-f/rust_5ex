# todo_1

 Version: 0.9.1

 date    : 2026/03/07
 
 update :

***

Rust , todo-app , Agent skills

* rustc 1.93.0 
* sqlite

***
## setup

***
* db-create

```
sqlite3 todo.db
```

***
* table.sql
```
CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    created_at TEXT,
    updated_at TEXT
);
```

***
* DB-Path
* src/main.rs , DB_PATH value change
```
static DB_PATH: &str = "sqlite:/home/user/todo_1/todo.db";
```

***
* list
```
target/debug/todo_1 todos
```

* add
```
target/debug/todo_1 todo_add todo-123
```

* delete
```
target/debug/todo_1 todo_delete 3
```
***
### blog


