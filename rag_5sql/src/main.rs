use anyhow::{Context};
use bytemuck::cast_slice;
use dotenvy::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use rusqlite::{ffi::sqlite3_auto_extension, Connection, Result, params};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
fn db_insert(db: &Connection, title: &str, content: &str, embedding: &Vec<f32>) -> Result<i64> {
    // 1. documents テーブルに本文を挿入
    let new_id = Uuid::new_v4();
    let blob = vec_f32_to_blob(embedding);
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

/// Vec<f32> を BLOB (バイト列) に変換
fn vec_f32_to_blob(vec: &Vec<f32>) -> Vec<u8> {
    vec.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn conver_u8_to_f32(data: Vec<u8>) -> Vec<f32>{
    let floats: &[f32] = cast_slice(&data);
    //println!("{:?}", floats);
    return floats.to_vec();
}

const TOP_K: usize = 3;

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
        let results : Vec<mod_search::SearchResult> = mod_search::db_search(&input_f32, TOP_K, query).await?;
        //println!("results.len={}" , results.len());
        return Ok(());
   }

  Ok(())
}