use anyhow::{Context};
use bytemuck::cast_slice;
use dotenvy::dotenv;
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use rusqlite::{ffi::sqlite3_auto_extension, Connection, Result, params};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
//use sqlite_vec::sqlite3_vec_init;
use zerocopy::AsBytes;
use std::env;
use std::fs;
use std::fmt;
use std::path::Path;
use std::io::{self, Read, Write};
use uuid::Uuid;

mod mod_search;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadParam{
    name:    String,
    content: String,
    embed:   String,
}

#[derive(Debug)]
struct VectorLengthError;

impl fmt::Display for VectorLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vectors must have the same length")
    }
}
impl std::error::Error for VectorLengthError {}

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
//fn db_insert(db: &Connection, title: &str, content: &str, embedding: &[f32]) -> Result<i64> {
fn db_insert(db: &Connection, title: &str, content: &str, embedding: &Vec<f32>) -> Result<i64> {
    // 1. documents テーブルに本文を挿入
    let new_id = Uuid::new_v4();
    let blob = vec_f32_to_blob(embedding);
    //let blob = vec_f32_to_blob(&embedding);
    println!("#blob.len={}" , blob.len());
    db.execute(
        "INSERT INTO document (id, content, embeddings) VALUES (?1, ?2, ?3)",
        params![new_id.to_string(), content, blob],
    )?;
    let doc_id = db.last_insert_rowid();

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

fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f64, Box<dyn std::error::Error>> {
    if a.len() != b.len() {
        return Err(Box::new(VectorLengthError));
    }

    let mut dot_product = 0.0_f64;
    let mut a_magnitude = 0.0_f64;
    let mut b_magnitude = 0.0_f64;

    for i in 0..a.len() {
        dot_product += (a[i] * b[i]) as f64;
        a_magnitude += (a[i] * a[i]) as f64;
        b_magnitude += (b[i] * b[i]) as f64;
    }

    if a_magnitude == 0.0 || b_magnitude == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (a_magnitude.sqrt() * b_magnitude.sqrt()))
}

/// Vec<f32> を BLOB (バイト列) に変換
fn vec_f32_to_blob(vec: &Vec<f32>) -> Vec<u8> {
    vec.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

/// BLOB (バイト列) を Vec<f32> に変換
fn blob_to_vec_f32(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}

fn conver_u8_to_f32(data: Vec<u8>) -> Vec<f32>{
    let floats: &[f32] = cast_slice(&data);
    //println!("{:?}", floats);
    return floats.to_vec();
}

/// KNN検索 → 上位K件のドキュメントを返す
async fn db_search(query_embedding: &[f32], k: usize, query: String) -> Result<Vec<SearchResult>> {
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    let conn = Connection::open(db_url)?;
    println!("#db_search-start");
    let items : Vec<SearchResult>  = Vec::new();

    #[derive(Debug)]
    struct VecData {
        id: String,
        content: String,
        embeddings: Vec<u8>
    }
    #[derive(Debug)]
    struct FloatData {
        id: String,
        content: String,
        embeddings: Vec<f32>
    }
    #[derive(Debug)]
    struct ScoreData {
        id: String,
        content: String,
        score: f64
    }    

    // ---- SELECT 全件取得 ----
    println!("\nSELECT 全件:");
    let mut stmt = conn.prepare(
        "SELECT id, name, content, embeddings FROM document",
    )?;

    let rows = stmt.query_map([], |row| {
        let id: String      = row.get(0)?;
        let name: String    = "".to_string();
        let content: String = row.get(2)?;
        let blob: Vec<u8>   = row.get(3)?;  // BLOB は Vec<u8> で受け取る
        Ok((id, name, content, blob))
    })?;

    let mut vecItems = Vec::new();
    for row in rows {
        let (id, name, content, blob) = row?;
        let embeddings = blob_to_vec_f32(&blob);
        //println!("  id={}, name={}, content={}, embeddings={:?}",
        //    id, name, content, embeddings);
        vecItems.push(FloatData {
            id,
            content,
            embeddings: embeddings,
        });
    }
    let mut scoreItems = Vec::<ScoreData>::new();
    for row_item in &vecItems {
        let distance = cosine_similarity(&query_embedding, &row_item.embeddings).unwrap();
        if distance > 0.6 {
            //println!("id={}, distance={} \n", row_item.id, distance);
            scoreItems.push(ScoreData {
                id: row_item.id.clone(),
                content: row_item.content.clone(),
                score: distance,
            });
        }
    }
    // score の降順ソート
    scoreItems.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut outItems = Vec::<ScoreData>::new(); 
    let top_k = 3;
    let mut outCount = 0;
    for row_item in &scoreItems {
        //println!("id={}, score={} \n",row_item.id,  row_item.score);
        if outCount < top_k {
            outItems.push(ScoreData {
                id: row_item.id.clone(),
                content: row_item.content.clone(),
                score: row_item.score,
            });            
        }
        outCount += 1;
    }
    let mut matches : String = "".to_string();
    for row_item in &outItems {
        println!("id={}, score={} \n",row_item.id,  row_item.score);
        let content_str = format!("{}\n\n", &row_item.content);
        matches.push_str(&content_str.clone().to_string());
    }
    let mut out_str : String = "".to_string();
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    } 
    let send_text = format!("日本語で、回答して欲しい。\n 要約して欲しい。\n\n{}", out_str);
    //let new_text = format!("要約して欲しい。\n\n {}", send_text);              
    println!("send_text={}\n", send_text);
    send_post(send_text).await;
 
    return Ok(items);
}

const TOP_K: usize = 3;

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

async fn GetEmbedding(api_key: &str, text: &str) -> anyhow::Result<Vec<f32>> {
    let embedding_values = mod_search::get_embedding(&api_key, &text).await.unwrap();
    Ok(embedding_values)
}

/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() ->Result<()> {
    dotenv().ok();
    // 環境変数取得
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("環境変数 GEMINI_API_KEY が設定されていません");

    let mut query = "二十四節気".to_string();

    let args: Vec<String> = env::args().collect();
    println!("arg.len={}" ,args.len());
    println!("実行パス: {}", args[0]);
    //unsafe {
    //    sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    //}

    let db = Connection::open(db_url)?;
    const VECTOR_DIM: usize = 3072;

    if args.len() == 2 && args[1] == "embed"{
        println!("#embed-start");
        let file_items = readTextData().unwrap();
        if file_items.len() == 0 {
            print!("error, file_items = 0");
            return Ok(());
        }        
        for row_file in &file_items {
            // ベクトルを取得
            let embedding_values = GetEmbedding(&api_key, &row_file.content).await.unwrap();

            println!("取得成功! ベクトル次元数: {}", embedding_values.len());            
            let id = db_insert(&db, "", &row_file.content.clone() , &embedding_values)?;
        }   
        return Ok(());
    }
    if args.len() == 3 && args[1] == "search"{
        query =args[2].clone();
        println!("search-mode.query: {}\n", query);
        let input_f32 = GetEmbedding(&api_key, &query).await.unwrap();

        println!("取得成功! ベクトル次元数: {}", input_f32.len());         
        let results : Vec<SearchResult> = db_search(&input_f32, TOP_K, query).await?;
        //println!("results.len={}" , results.len());
        return Ok(());
   }

  Ok(())
}