# rag_5sql

 Version: 0.9.1

 date    : 2026/06/05
 
 update :

***

Rust RAG Search + SQLite database

* embedding: gemini-embedding-001
* model: gemma-4-E2B
* llama.cpp , llama-server use
* rustc 1.93.0 

***
## setup

* llama-server start
* port 8090: gemma-4-E2B

```
#gemma-4-E2B
/usr/local/llama-b8642/llama-server -m /var/lm_data/unsloth/gemma-4-E2B-it-Q4_K_S.gguf \
 --chat-template-kwargs '{"enable_thinking": false}' --port 8090 

```

***
### related
https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF

***
* table add
```
sqlite3 ./example.db < table.sql
```

***
### build

```
cargo build
```

***
* env value, GEMINI-API etc
```
SET DATABASE_URL=example.db
SET GEMINI_API_KEY=
```
***
* embed
```
target\debug\rag_5sql.exe embed
```

* RAG search
```
target\debug\rag_5sql.exe search hello
```

***
### blog

https://zenn.dev/knaka0209/scraps/16a42f2c0edfe0

