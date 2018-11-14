extern crate a03;
extern crate rusqlite;

use a03::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

fn main() {
	let conn = Connection::open("dfs.db").unwrap();
}

#[derive(Debug)]
struct Dnode {
    id : i32,
    address : String,
    port : i32,
}

fn create_tables(conn : &Connection) {
	conn.execute(
    	"CREATE TABLE dnode (nid INTEGER PRIMARY KEY ASC AUTOINCREMENT, address TEXT NOT NULL DEFAULT \" \", port INTEGER NOT NULL DEFAULT \"0\")",
    	NO_PARAMS);
}

fn add_data_node(conn : &Connection, address : &str, port : i32) {
    let dn = Dnode {
        id : 0,
        address : address.to_string(),
        port,
    };
    match conn.execute(
        "INSERT INTO dnode (address, port) VALUES (?1, ?2)",
        &[&dn.address as &ToSql, &dn.port]) {
        Ok(n) => println!("{} rows updated", n),
        Err(e) => println!("INSERT error: {}", e)
    }
}

fn check_node(conn : Connection, address : String, port : usize) {
}

fn get_data_nodes(conn : Connection) {
}

fn insert_file(conn : Connection, fname : String, fsize : usize) {
}

fn get_file_info(conn : Connection, fname : String) {
}

fn get_files(conn : Connection) {
}

fn add_block_to_inode(conn : Connection, fname : String, blocks : usize) {
}

fn get_file_inode(conn : Connection, fname : String) {
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn);
        conn
    }

    #[test]
    fn inserts_dnode() {
        let conn = get_test_db();
        add_data_node(&conn, "127.0.0.1", 65533);
        let mut stmt = conn.prepare("SELECT * FROM dnode").unwrap();
        let result = stmt.query_map(
            NO_PARAMS, |row| Dnode {
                id : row.get(0), address : row.get(1), port : row.get(2),
            }).unwrap();
        for d in result {
            let dnode = d.unwrap();
            assert_eq!(dnode.id, 1);
            assert_eq!(dnode.address, "127.0.0.1");
            assert_eq!(dnode.port, 65533);
        }
    }

}
