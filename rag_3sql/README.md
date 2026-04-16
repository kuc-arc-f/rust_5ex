# rag_3sql

 Version: 0.9.1

 date    : 2026/04/17
 
 update :

***

Rust Server (axum) RAG Search + sqlite-vec

* sqlite-vec Database
* model: gemma-4-E2B
* embedding: qwen3-embedding:0.6b , llama.cpp
* llama.cpp , llama-server use
* anum
* rustc 1.93.0 

***
### vector data add

https://github.com/kuc-arc-f/rust_5ex/tree/main/sqlite-vec-2

***
## setup

* llama-server start
* port 8080: Qwen3-Embedding-0.6B
* port 8090: gemma-4-E2B

```
#Qwen3-Embedding-0.6B
/home/user123/llama-server -m /var/lm_data/Qwen3-Embedding-0.6B-Q8_0.gguf --embedding  -c 1024 --port 8080

#gemma-4-E2B
/usr/local/llama-b8642/llama-server -m /var/lm_data/unsloth/gemma-4-E2B-it-Q4_K_S.gguf \
 --chat-template-kwargs '{"enable_thinking": false}' --port 8090 

```
***
### database

./db.sqlite

***
### related
https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF

https://huggingface.co/Qwen/Qwen3-Embedding-0.6B-GGUF

***
* front build

```
npm run build
```

***
### build

```
cargo build
```

***
* RAG search

```
target/debug/rag_server
```

***
### blog

