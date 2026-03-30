# sqlite-vec-3

 Version: 0.9.1

 date    : 2026/03/28
 
 update :

***

Rust , RAG Search + sqlite-vec

* sqlite-vec Database
* model: Qwen3.5-2B
* embedding: qwen3-embedding:0.6b , llama.cpp
* llama.cpp , llama-server use
* rustc 1.93.0 

***
## setup

* llama-server start
* port 8080: Qwen3-Embedding-0.6B
* port 8090: Qwen3.5-2B

```
#Qwen3-Embedding-0.6B
/home/user123/llama-server -m /var/lm_data/Qwen3-Embedding-0.6B-Q8_0.gguf --embedding  -c 1024 --port 8080

#Qwen3.5-2B
/home/user123/llama-server -m /var/lm_data/unsloth/Qwen3.5-2B-GGUF/Qwen3.5-2B-Q4_K_S.gguf \
 --chat-template-kwargs '{"enable_thinking": false}' --port 8090 

```

***
### related
https://huggingface.co/unsloth/Qwen3.5-2B-GGUF

https://huggingface.co/Qwen/Qwen3-Embedding-0.6B-GGUF


***
* env value

```
export DATABASE_URL=db.sqlite
```

***
* build
```
cargo build
```

***
* init, table
```
target/debug/sqlite-vec-3 init
```

***
* vector data add

```
target/debug/sqlite-vec-3 embed
```

***
* RAG search

```
target/debug/sqlite-vec-3 search hello
```

***
* data:  text file
```
data path: ./data
```

***
### blog

