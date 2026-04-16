use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
    response::{Html, IntoResponse, Json},
};
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use rusqlite::{ffi::sqlite3_auto_extension, Connection, Result, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlite_vec::sqlite3_vec_init;
use std::env;
use std::io::{self, Read};
use tower_http::services::ServeDir;
use uuid::Uuid;
use zerocopy::AsBytes;

mod mod_search;

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
    content: Option<String>,
}
/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() {
    // `public` フォルダのパス
    let public_dir = "public/static";

    // `ServeDir` ミドルウェアを初期化
    let serve_dir = ServeDir::new(public_dir);

    let app = Router::new()
        .nest_service("/static", serve_dir)
        .route("/api/rag_search", post(rag_search))
        .route("/foo", get(get_foo))
        .route("/", get(root))
        .route("/*path", get(root))
        ;
    println!("Listening on http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    let s1 = "<!doctype html>
<html>
  <head>
    <meta charset='UTF-8' />
    <meta name='viewport' content='width=device-width, initial-scale=1.0' />
    <title>welcome</title>
    <script src='https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4'></script>
  </head>
  <body>
    <div id='app'></div>
    <script type='module' src='/static/client.js'></script>
  <body>
</html>
";
  Html(&s1)
}

async fn get_foo() -> String {
    String::from("foo\n")
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryReq {
    input: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResp {
    ret: String,
    text: String,
} 
#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    id:       i64,
    title:    String,
    content:  String,
    source:   Option<String>,
    distance: f64,
}

static DB_PATH: &str = "./db.sqlite";
const TOP_K: usize = 3;

fn db_search(query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
    let db = Connection::open(DB_PATH)?;
    println!("#db_search-start");
    let items : Vec<SearchResult>  = Vec::new();

    //結合SQL
    let mut stmt = db.prepare(
    r"SELECT d.id, d.title, d.content, d.source, v.distance
        FROM doc_vectors v
        JOIN documents d ON d.id = v.rowid
        WHERE v.embedding MATCH ?1 AND k = 1
        ORDER BY v.distance
        ",
    )?;
    let vec_items = stmt.query_map(params![query_embedding.as_bytes()], |row| {
        Ok(SearchResult {
            id:    row.get(0)?,
            title: "".to_string(),
            content: row.get(2)?,
            source: None,
            distance:  row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;    

    //println!("{:?}" ,vec_items);
    return Ok(vec_items);

    Ok(items)
}

/**
*
* @param
*
* @return
*/
async fn send_post(input : String) -> String
{
   #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<Message>,
        temperature: f32,
    }
    #[derive(Debug, Deserialize)]
    struct ChatResponse {
        choices: Vec<Choice>,
    }

    #[derive(Debug, Deserialize)]
    struct Choice {
        message: MessageContent,
    }

    #[derive(Debug, Deserialize)]
    struct MessageContent {
        role: String,
        content: String,
    }
    let client = Client::new();
    let request_body = ChatRequest {
        model: "qwen3.5-2b".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: input.to_string(),
            }
        ],
        temperature: 0.7,
    };
    let response = client
        .post("http://localhost:8090/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await.unwrap();

    let result: ChatResponse = response.json().await.unwrap();

    let mut resp_str = "".to_string();
    if let Some(choice) = result.choices.first() {
        resp_str = choice.message.content.clone();
        println!("AI: {}", choice.message.content);
    } 
    return resp_str;
}
/**
*
* @param
*
* @return
*/
pub async fn rag_search(
    Json(payload): Json<QueryReq>,
) -> Result<Json<SearchResp>, StatusCode> {
    println!("#rag_search");
    println!("input: {}", &payload.input);
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }
    let db = Connection::open(DB_PATH).unwrap();
    let mut query = &payload.input.clone();

    const VECTOR_DIM: usize = 1024;

    println!("search-mode.query: {}\n", query);
    let input_f32 = mod_search::EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());
    let results : Vec<SearchResult> = db_search(&input_f32, TOP_K).unwrap();
    println!("results.len={}" , results.len());

    let mut matches : String = "".to_string();
    let mut out_str : String = "".to_string();
    if results.len() > 0 {
        let target_dim = &results[0];
        println!("id={}" , target_dim.id);
        let distance = target_dim.distance;
        println!("distance={}" , distance);
        let content_str = format!("{}\n\n", &target_dim.content);
        if(distance < 1.0) {
            matches.push_str(&content_str.clone().to_string());
        }
    }
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    } 
    //println!("out_str={}" , out_str);
    let send_text = format!("日本語で、回答して欲しい。\n{}", out_str);
    let new_text = format!("要約して欲しい。\n\n {}", send_text);              
    println!("new_text={}\n", new_text);
    let resp = send_post(new_text).await;

    let resp_data = SearchResp {
      ret: "OK".to_string(),
      text: resp.to_string(),
    };

    Ok(Json(resp_data))
}

