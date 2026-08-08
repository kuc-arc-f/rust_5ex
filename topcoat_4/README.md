# topcoat_4

 Version: 0.9.1

 date    : 2026/08/08
 
 update :

***

Rust topcoat , API TODO example

* json file , save data

***
### related

https://github.com/tokio-rs/topcoat/blob/main/crates/topcoat/docs/getting_started.md

***
* topcoat-cli , install
```
cargo install topcoat-cli
```

* topcoat-start, localhost:3000 start
```
topcoat dev
```

***
* test-code
* add
```
curl -X POST -H "Content-Type: application/json" \
 -d '{"title": "TEST-DATA-001"}' \
 http://localhost:3000/api/todo/create
```

* list
```
curl http://localhost:3000/api/todo/list
```

***
### blog

https://zenn.dev/knaka0209/scraps/ff302e1902b488

