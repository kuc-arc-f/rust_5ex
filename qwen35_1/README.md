# qwen35_1

 Version: 0.9.1

 date    : 2026/03/04
 
 update :

***

Rust , RAG Search

* qdrant Db use
* model: Qwen3.5-2B
* embedding: qwen3-embedding:0.6b , ollama
* llama.cpp , llama-server use
* rustc 1.93.0 

***
## setup

* init, collection add
```
target\debug\qwen35_1.exe init
```

***
* vector data add

```
target\debug\qwen35_1.exe create
```

***
* RAG search

```
target\debug\qwen35_1.exe search hello
```

***
* data:  text file
```
data path: ./data
```

***
