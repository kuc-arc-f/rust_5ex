# sqlite-vec-5

 Version: 0.9.1

 date    : 2026/05/31
 
 update :

***

Rust , sqlite-vec RAG example

* sqlite-vec Database
* rustc 1.93.0 
* embedding: gemini-embedding-001

***
## setup

```
export DATABASE_URL=/home/user123/sqlite-vec-5/db.sqlite
export GEMINI_API_KEY=
```

***

* build
```
cargo build
```

***
* init, table
```
target/debug/sqlite-vec-5 init
```

***
* vector data add

```
target/debug/sqlite-vec-5 embed
```

***
* RAG search

```
target/debug/sqlite-vec-5 search hello
```

***

### blog


