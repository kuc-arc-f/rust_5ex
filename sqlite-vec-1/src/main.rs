use rusqlite::{ffi::sqlite3_auto_extension, Connection, Result};
use sqlite_vec::sqlite3_vec_init;
use zerocopy::AsBytes;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("arg.len={}" ,args.len());
    println!("実行パス: {}", args[0]);
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    //let db = Connection::open_in_memory()?;
    let db = Connection::open("./db.sqlite")?;
    let v: Vec<f32> = vec![0.1, 0.2, 0.3];

    let (sqlite_version, vec_version, x): (String, String, String) = db.query_row(
        "select sqlite_version(), vec_version(), vec_to_json(?)",
        &[v.as_bytes()],
        |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?)),
    )?;

    println!("sqlite_version={sqlite_version}, vec_version={vec_version}");

    let items: Vec<(usize, Vec<f32>)> = vec![
        (1, vec![0.1, 0.1, 0.1, 0.1]),
        (2, vec![0.2, 0.2, 0.2, 0.2]),
        (3, vec![0.3, 0.3, 0.3, 0.3]),
        (4, vec![0.4, 0.4, 0.4, 0.4]),
        (5, vec![0.5, 0.5, 0.5, 0.5]),
    ];
    println!("{x}");
    if args.len() == 2 && args[1] == "init"{
        println!("#init-start");
        //CREATE
        db.execute(
            "CREATE VIRTUAL TABLE vec_items USING vec0(embedding float[4])",
            [],
        )?;
        return Ok(());
    }
    if args.len() == 2 && args[1] == "embed"{
        println!("#embed-start");
        //INSERT
        let mut stmt = db.prepare("INSERT INTO vec_items(rowid, embedding) VALUES (?, ?)")?;
        for item in items {
            stmt.execute(rusqlite::params![item.0, item.1.as_bytes()])?;
        }
        return Ok(());
    }
    if args.len() == 2 && args[1] == "search"{
        //SELECT
        let query: Vec<f32> = vec![0.3, 0.3, 0.3, 0.3];
        let result: Vec<(i64, f64)> = db
            .prepare(
                r"
            SELECT
                rowid,
                distance
            FROM vec_items
            WHERE embedding MATCH ?1
            ORDER BY distance
            LIMIT 3
            ",
            )?
            .query_map([query.as_bytes()], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        println!("{:?}", result);
    }


    Ok(())
}