use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use rusqlite::{ffi::sqlite3_auto_extension, Connection, Result, params};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlite_vec::sqlite3_vec_init;
use zerocopy::AsBytes;
use std::env;
use std::fs;
use std::path::Path;
use std::io::{self, Read};
use uuid::Uuid;

mod mod_search;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadParam{
    name:    String,
    content: String,
    embed:   String,
}

/**
*
* @param
*
* @return
*/
fn readTextData()-> anyhow::Result<Vec<ReadParam>, String> {
    // 読み込み対象のフォルダパスを指定
    let folder_path = Path::new("./data/");
    let mut read_items: Vec<ReadParam> = Vec::new();
    let mut row_file_name :String= "".to_string();
    let mut row_file_cont :String = "".to_string();

    // フォルダが存在するか確認
    if !folder_path.is_dir() {
        // 存在しない場合は作成するか、処理を終了する
        eprintln!("エラー: フォルダ '{}' が存在しません。", folder_path.display());
        return Err("error, folder none".to_string()); 
    }

    println!("--- フォルダ: {} 内の .txt ファイルを読み込みます ---", folder_path.display());

    // フォルダ内のエントリをイテレート
    for entry in fs::read_dir(folder_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // ファイルであり、拡張子が ".txt" または ".md" であることを確認
        if path.is_file() && path.extension().map_or(false, |ext| ext == "txt" || ext == "md") {
            println!("\n[ファイル: {}]", path.display());
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(); // OsStrをStringに変換（エラーを無視）

            println!("filename={}", filename); 
            row_file_name = filename.to_string();
            
            // ファイルを開く
            match fs::File::open(&path) {
                Ok(mut file) => {
                    // ファイルの内容を保持するためのString
                    let mut contents = String::new();
                    
                    // ファイル全体を文字列に読み込む
                    match file.read_to_string(&mut contents) {
                        Ok(_) => {
                            // 読み込んだ内容を出力
                            println!("内容:\n{}", contents);
                            row_file_cont = contents.to_string();
                            println!("\n=== 再帰的分割 ===");
                            read_items.push(ReadParam{
                                name: row_file_name.clone(),
                                content: row_file_cont.clone(),
                                embed: "".to_string(),
                            })                                
                        },
                        Err(e) => {
                            eprintln!("エラー: ファイル '{}' の読み込み中にエラーが発生しました: {}", path.display(), e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("エラー: ファイル '{}' を開けませんでした: {}", path.display(), e);
                }
            }
        }
    }
    //println!("{:?}", read_items);
    println!("--- 読み込み完了 ---");
    return Ok(read_items);
}
/// ドキュメントとベクトルをトランザクションで登録
fn db_insert(db: &Connection, title: &str, content: &str, embedding: &[f32]) -> Result<i64> {
    // 1. documents テーブルに本文を挿入
    db.execute(
        "INSERT INTO documents (title, content) VALUES (?1, ?2)",
        params![title, content],
    )?;
    let doc_id = db.last_insert_rowid();
    println!("doc_id={}", doc_id);
    // 2. 同じ rowid でベクトルを挿入
    db.execute(
        "INSERT INTO doc_vectors (rowid, embedding) VALUES (?1, ?2)",
        params![doc_id, embedding.as_bytes()],
    )?;

    Ok(doc_id)
}
// ================================================================
// 検索結果の型
// ================================================================

#[derive(Debug)]
struct SearchResult {
    id:       i64,
    title:    String,
    content:  String,
    source:   Option<String>,
    distance: f64,
}

/// KNN検索 → 上位K件のドキュメントを返す
fn db_search(query_embedding: &[f32], k: usize) -> Result<Vec<SearchResult>> {
    let db = Connection::open(DB_PATH)?;
    println!("#db_search-start");
    let items : Vec<SearchResult>  = Vec::new();
    /*
    let items : Vec<SearchResult>  = Vec::new();
    let mut stmt = db.prepare(
    r"
        SELECT
            rowid,
            distance
        FROM doc_vectors
        WHERE embedding MATCH ?1
        ORDER BY distance
        LIMIT 3
        ",
    )?;
    let vec_items = stmt.query_map(params![query_embedding.as_bytes()], |row| {
        Ok(SearchResult {
            id:    row.get(0)?,
            title: "".to_string(),
            content: "".to_string(),
            source: None,
            distance:  row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?; 
    println!("{:?}" ,vec_items);   
    */

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

const TOP_K: usize = 3;
static DB_PATH: &str = "./db.sqlite";

/**
*
* @param
*
* @return
*/
async fn send_post(input : String) {
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

    if let Some(choice) = result.choices.first() {
        println!("AI: {}", choice.message.content);
    } 
}
/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() -> Result<()> {
    let mut query = "二十四節気".to_string();

    let args: Vec<String> = env::args().collect();
    println!("arg.len={}" ,args.len());
    println!("実行パス: {}", args[0]);
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    let db = Connection::open(DB_PATH)?;
    let v: Vec<f32> = vec![0.1, 0.2, 0.3];

    let (sqlite_version, vec_version, x): (String, String, String) = db.query_row(
        "select sqlite_version(), vec_version(), vec_to_json(?)",
        &[v.as_bytes()],
        |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?)),
    )?;

    println!("sqlite_version={sqlite_version}, vec_version={vec_version}");
    const VECTOR_DIM: usize = 1024;
    //init
    if args.len() == 2 && args[1] == "init"{
        println!("#init-start");
        db.execute_batch(&format!(
            "-- 文章本文・メタデータを保持するテーブル
             CREATE TABLE IF NOT EXISTS documents (
                 id      INTEGER PRIMARY KEY AUTOINCREMENT,
                 title   TEXT NOT NULL,
                 content TEXT NOT NULL,
                 source  TEXT
             );

             -- ベクトルインデックス（sqlite-vec仮想テーブル）
             CREATE VIRTUAL TABLE IF NOT EXISTS doc_vectors
                 USING vec0(embedding float[{VECTOR_DIM}]);
            "
        ))?;        
        return Ok(());
    }
    //embed
    if args.len() == 2 && args[1] == "embed"{
        println!("#embed-start");
        let file_items = readTextData().unwrap();
        if file_items.len() == 0 {
            print!("error, file_items = 0");
            return Ok(());
        }        
        for row_file in &file_items {
            let input_f32 = mod_search::EmbedUserQuery(row_file.content.clone()).await;
            println!("input_f32.len={}", input_f32.len());
            let id = db_insert(&db, "", &row_file.content.clone() , &input_f32)?;
        }   
        return Ok(());
    }
    if args.len() == 3 && args[1] == "search"{
        query =args[2].clone();
        println!("search-mode.query: {}\n", query);
        let input_f32 = mod_search::EmbedUserQuery(query.clone()).await;
        println!("input_f32.len={}", input_f32.len());

        let results : Vec<SearchResult> = db_search(&input_f32, TOP_K)?;
        println!("results.len={}" , results.len());

        let mut matches : String = "".to_string();
        let mut out_str : String = "".to_string();
        if results.len() > 0 {
            let target_dim = &results[0];
            println!("id={}" , target_dim.id);
            println!("distance={}" , target_dim.distance);
            let content_str = format!("{}\n\n", &target_dim.content);
            matches.push_str(&content_str.clone().to_string());
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
        send_post(new_text).await;
    }

    Ok(())
}