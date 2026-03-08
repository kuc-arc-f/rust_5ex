use serde::{Serialize, Deserialize};
use sqlx::{FromRow, SqlitePool};
use std::env;

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Todo {
    id: i64,
    title: String,
    //content: Option<String>,
}

static DB_PATH: &str = "sqlite:/home/naka/work/rust/extra/todo_1/todo.db";

/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(DB_PATH).await?;
    // 引数をベクターとして収集
    let args: Vec<String> = env::args().collect();
    //println!("実行パス: {}", args[0]);
    //println!("cmd: {}", args[1]);
    //println!("arg.len=: {}", args.len());

    if args.len() == 3 && args[1] == "todo_add" {
        //println!("#todo_add-start");
        let input = args[2].clone();
        insert_todo(&pool, &input, "test content").await?;
        let count = count_todo(&pool).await?;
        //println!("COUNT {}", count);
        let list = select_all(&pool).await;
        return Ok(());
    }
    if args.len() == 3 && args[1] == "todo_delete" {
        let input = args[2].clone();
        let mut target : i32 = 0;
        if let Ok(num) = input.parse::<i32>() {
            println!("num={}", num);
            target = num;
        }        
        delete_todo(&pool, target.into()).await?;
        let list = select_all(&pool).await;
        return Ok(());
    }
    if args[1] == "todos" {
        let list = select_all(&pool).await;
        return Ok(());
    }
    Ok(())
}

/**
*
* @param
*
* @return
*/
async fn insert_todo(
    pool: &SqlitePool,
    title: &str,
    content: &str,
) -> Result<(), sqlx::Error> {

    sqlx::query(
        "INSERT INTO todos (title) VALUES (?)"
    )
    .bind(title)
    .execute(pool)
    .await?;

    Ok(())
}

/**
*
* @param
*
* @return
*/
async fn select_all(pool: &SqlitePool) -> String {
    let rows = sqlx::query_as::<_, Todo>(
        "SELECT id, title FROM todos"
    )
    .fetch_all(pool)
    .await.unwrap();

    let mut todos: Vec<Todo> = Vec::new();
    for v in &rows {
        todos.push(Todo {
            id: v.id,
            title: v.title.clone(),
        })
    }
    // JSON変換
    let json = serde_json::to_string(&todos);
    println!("{:?}", json);
    "".to_string()
}
/**
*
* @param
*
* @return
*/
async fn update_todo(
    pool: &SqlitePool,
    id: i64,
    title: &str
) -> Result<(), sqlx::Error> {

    sqlx::query(
        "UPDATE todos SET title=? WHERE id=?"
    )
    .bind(title)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/**
*
* @param
*
* @return
*/
async fn delete_todo(
    pool: &SqlitePool,
    id: i32
) -> Result<(), sqlx::Error> {

    sqlx::query(
        "DELETE FROM todos WHERE id=?"
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}
/**
*
* @param
*
* @return
*/
async fn count_todo(pool: &SqlitePool) -> Result<i64, sqlx::Error> {

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM todos")
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}