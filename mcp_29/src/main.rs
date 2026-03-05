use dotenvy::dotenv;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, ScalarQuantizationBuilder,
    SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant, QdrantError};
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::env;
use std::fs;
use std::path::Path;
use std::io::{self, Read};

use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

mod mod_search;
static COLLECT_NAME: &str = "document-3";
static EMBED_SIZE: u64 =1024;

/// テキストをチャンクに分割する構造体
pub struct TextSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadParam{
    name:    String,
    content: String,
    embed:   String,
}

impl TextSplitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        assert!(chunk_overlap < chunk_size, "オーバーラップはチャンクサイズより小さくする必要があります");
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    /// 基本的な文字ベース分割
    pub fn split_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<&str> = text.graphemes(true).collect();
        
        let mut start = 0;
        while start < chars.len() {
            let end = std::cmp::min(start + self.chunk_size, chars.len());
            let chunk: String = chars[start..end].iter().copied().collect();
            chunks.push(chunk);
            
            if end >= chars.len() {
                break;
            }
            start += self.chunk_size - self.chunk_overlap;
        }
        
        chunks
    }

    /// 再帰的分割（段落 -> 文 -> 単語の順）
    pub fn recursive_split(&self, text: &str) -> Vec<String> {
        let separators = vec!["\n\n", "\n", "。", ".", " "];
        self.recursive_split_with_separators(text, &separators)
    }

    fn recursive_split_with_separators(&self, text: &str, separators: &[&str]) -> Vec<String> {
        if separators.is_empty() {
            return self.split_text(text);
        }

        let separator = separators[0];
        let remaining_separators = &separators[1..];
        
        let parts: Vec<&str> = text.split(separator).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for part in parts {
            let part_len = part.graphemes(true).count();
            let current_len = current_chunk.graphemes(true).count();

            if current_len + part_len + separator.len() <= self.chunk_size {
                if !current_chunk.is_empty() {
                    current_chunk.push_str(separator);
                }
                current_chunk.push_str(part);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }

                if part_len > self.chunk_size {
                    // さらに細かく分割
                    let sub_chunks = self.recursive_split_with_separators(part, remaining_separators);
                    chunks.extend(sub_chunks);
                } else {
                    current_chunk = part.to_string();
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// センテンス単位での分割
    pub fn split_by_sentences(&self, text: &str) -> Vec<String> {
        let sentences: Vec<&str> = text
            .split(|c| c == '。' || c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in sentences {
            let sentence_with_period = format!("{}。", sentence.trim());
            let sentence_len = sentence_with_period.graphemes(true).count();
            let current_len = current_chunk.graphemes(true).count();

            if current_len + sentence_len <= self.chunk_size {
                current_chunk.push_str(&sentence_with_period);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                }
                current_chunk = sentence_with_period;
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

/**
*
* @param
*
* @return
*/
fn readTextData()-> anyhow::Result<Vec<ReadParam>, String> {
    let splitter = TextSplitter::new(500, 100);
    // 読み込み対象のフォルダパスを指定
    let folder_path = Path::new("./data/");
    let mut read_items: Vec<ReadParam> = Vec::new();
    let mut row_file_name :String= "".to_string();
    let mut row_file_cont :String = "".to_string();

    // フォルダが存在するか確認
    if !folder_path.is_dir() {
        // 存在しない場合は作成するか、処理を終了する
        eprintln!("エラー: フォルダ '{}' が存在しません。", folder_path.display());
        // 以下の行をコメントアウトしてフォルダ作成処理を追加しても良い
        // fs::create_dir_all(folder_path)?;
        return Err("error, folder none".to_string()); 
    }

    println!("--- フォルダ: {} 内の .txt ファイルを読み込みます ---", folder_path.display());

    // フォルダ内のエントリをイテレート
    for entry in fs::read_dir(folder_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // ファイルであり、拡張子が ".txt" であることを確認

        if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
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
                            let recursive_chunks = splitter.recursive_split(&row_file_cont);
                            for (i, chunk) in recursive_chunks.iter().enumerate() {
                                println!("チャンク {}: {}", i + 1, chunk);
                                read_items.push(ReadParam{
                                    name: row_file_name.clone(),
                                    content: chunk.clone(),
                                    embed: "".to_string(),
                                })                                
                            }
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

// Gemini APIのレスポンス構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    //pub embeddings: Vec<EmbeddingData>,
    embedding: EmbeddingData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

// エンベディング結果を格納する構造体
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub text: String,
    pub embedding: Vec<f32>,
}

/**
*
* @param
*
* @return
*/
async fn create_collection(client: &Qdrant, dim: u64) -> Result<(), QdrantError>{
    client
        .create_collection(
            CreateCollectionBuilder::new(COLLECT_NAME)
                .vectors_config(VectorParamsBuilder::new(dim, Distance::Cosine))
                .quantization_config(ScalarQuantizationBuilder::default()),
        )
        .await?;

    Ok(())
}
pub async fn EmbedUserQuery(query :String) -> Vec<f32> {
    #[derive(Deserialize, Debug)]
    struct EmbedResponse {
        embedding: Vec<f32>,
    }
    #[derive(Serialize)]
    struct EmbeddingRequest {
        model: String,
        prompt: String,
    }    

    let items : Vec<f32> = Vec::new();
    let client = reqwest::Client::new();

    let request = EmbeddingRequest {
        model: "qwen3-embedding:0.6b".to_string(),
        prompt: query.to_string(),
    };

    let res = client
        .post("http://localhost:11434/api/embeddings")
        .json(&request)
        .send()
        .await.unwrap();
    println!("Status: {:?}", res.status());

    if res.status().is_success() {
        let response_body: EmbedResponse = res.json().await.unwrap();
        println!("Embedding length: {}", response_body.embedding.len());
        // Print first few elements to verify without flooding console
        if response_body.embedding.len() > 0 {
             println!("dimensions.len: {}", response_body.embedding.len());
             return response_body.embedding;
         } else {
             println!("Embedding: {:?}", response_body.embedding);
        }
       
    } else {
        println!("Request failed: {:?}", res.status());
        let text = res.text().await.unwrap();
        println!("Response text: {}", text);
    }

   return items;
}
/**
*
* @param
*
* @return
*/
#[tokio::main]
async fn main() -> Result<(), QdrantError> {
    #[derive(Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
    }
    #[derive(Serialize)]
    struct OllamaRequestOprion {
        think: bool,
    }

    #[derive(Deserialize, Debug)]
    struct OllamaResponse {
        response: String,
    }    

    dotenv().ok();
    let query = "二十四節気".to_string();

    // 引数をベクターとして収集
    let args: Vec<String> = env::args().collect();
    // 注意: args[0] は実行ファイルのパスが入ります
    println!("実行パス: {}", args[0]);
    let clientQdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();

    if args.len() > 1 {
        println!("第一引数: {}", args[1]);
        if args[1].len() > 0 {
            if args[1] == "init" {
                create_collection(&clientQdrant, EMBED_SIZE).await.unwrap();
                println!("#init-start");
            } else if args[1] == "search" {
                println!("search-mode.query: {}\n", query);
                let resp = mod_search::CheckSimalirity(query).await;
                let send_text = format!("日本語で、回答して欲しい。\n{}", resp);
                //println!("send_text={}\n", send_text);
                let new_text = format!("以下のルールを必ず守ってください。\n <think> タグや思考過程は一切出力しない\n\n {}", send_text);
                println!("new_text={}\n", new_text);

                let client = Client::new();
                let req_opt = OllamaRequestOprion {
                    think: false
                };

                let body = OllamaRequest {
                    model: "lfm2.5-thinking:latest".to_string(),
                    prompt: new_text.to_string(),
                    stream: false,
                };

                let res = client
                    .post("http://localhost:11434/api/generate")
                    .json(&body)
                    .send()
                    .await.unwrap()
                    .json::<OllamaResponse>()
                    .await.unwrap();

                //println!("AI: {}", res.response); 
                let no_think_str = remove_think_tags(&res.response);
                println!("no_think: {}", no_think_str);                              

                return Ok(());
            }
        } else {
            println!("引数がありません");
            return Ok(());
        }
    }
    if args[1] != "create" {
        println!("not, create-mode");
        return Ok(());
    }    

    let file_items = readTextData().unwrap();
    if file_items.len() == 0 {
        print!("error, file_items = 0");
        return Ok(());
    }
    //println!("{:?}", file_items[0]);

    let mut cont_Items: Vec<String> = Vec::new(); 

    for row_file in &file_items {
        let input_f32 = EmbedUserQuery(row_file.content.clone()).await;
        println!("input_f32.len={}", input_f32.len());
        let payload: Payload = serde_json::json!(
            {
                "content": &row_file.content.clone()
            }
        )
        .try_into()
        .unwrap();

        let newUID = Uuid::new_v4();
        println!("new_id_str={}", &newUID);
        let points = vec![PointStruct::new(newUID.to_string(), input_f32, payload)];

        clientQdrant
            .upsert_points(UpsertPointsBuilder::new(COLLECT_NAME, points))
            .await?;           
    }   

    Ok(())         
}
fn remove_think_tags(text: &str) -> String {
    let re = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
    re.replace_all(text, "").trim().to_string()
}

/**
*
* @param
*
* @return
*/
fn f32_vec_to_u8_vec(data: &Vec<f32>) -> &[u8] {
    let len = data.len() * std::mem::size_of::<f32>();

    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            len,
        )
    }
}
/**
*
* @param
*
* @return
*/
fn print_type_of<T>(_: &T) {
    println!("Type: {}", std::any::type_name::<T>());
}
