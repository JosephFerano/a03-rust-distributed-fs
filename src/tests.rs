#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE inode (
fid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
fname TEXT UNIQUE NOT NULL DEFAULT \" \",
fsize INTEGER NOT NULL default \"0\")",
            NO_PARAMS
        ).unwrap();

        conn.execute(
            "CREATE TABLE dnode (
nid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
address TEXT NOT NULL DEFAULT \" \",
port INTEGER NOT NULL DEFAULT \"0\")",
            NO_PARAMS,
        ).unwrap();

        conn.execute(
            "CREATE TABLE block (
bid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
fid INTEGER NOT NULL DEFAULT \"0\",
nid INTEGER NOT NULL DEFAULT \"0\",
cid TEXT NOT NULL DEFAULT \"0\")",
            NO_PARAMS
        ).unwrap();
        conn
    }

    #[test]
    fn add_dnode_with_correct_ip_and_port() {
        let conn = get_test_db();
        let ip = "127.0.0.1";
        let port = 65533;
        add_data_node(&conn, &ip, port as i32);
        let dnode = get_data_node(&conn, &ip, port as i32).unwrap();
        assert_eq!(dnode.id, 1);
        assert_eq!(dnode.ip, ip);
        assert_eq!(dnode.port, port);
    }

    #[test]
    fn removes_dnode_with_correct_ip_and_port() {
        let conn = get_test_db();
        let ip = "127.0.0.1";
        let port = 65533;
        let dnode = get_data_node(&conn, &ip, port as i32);
        assert_eq!(dnode.is_none(), true);
        add_data_node(&conn, &ip, port as i32);
        let dnode = get_data_node(&conn, &ip, port as i32).unwrap();
        assert_eq!(dnode.ip, ip);
        assert_eq!(dnode.port, port);
        remove_data_node(&conn, &ip, port as i32);
        let dnode = get_data_node(&conn, &ip, port as i32);
        assert_eq!(dnode.is_none(), true);
    }

    #[test]
    fn gets_all_data_nodes() {
        let conn = get_test_db();
        let ip1 = "127.0.0.1";
        let port1 = 65533;
        let ip2 = "127.0.0.2";
        let port2 = port1 + 1;
        let ip3 = "127.0.0.3";
        let port3 = port2 + 1;
        add_data_node(&conn, &ip1, port1 as i32);
        add_data_node(&conn, &ip2, port2 as i32);
        add_data_node(&conn, &ip3, port3 as i32);
        let ds = get_data_nodes(&conn);
        for i in 0..ds.len() {
            let d = &ds[i];
            assert_eq!(d.id, (i + 1) as u32);
            assert_eq!(d.ip, format!("127.0.0.{}", i + 1));
            assert_eq!(d.port, 65533 + i as u32);
        }
    }

    #[test]
    fn adds_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("my_1337_virus"), 32);
        let files = get_files(&conn);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "my_1337_virus 32 bytes");
    }

    #[test]
    fn removes_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("my_1337_virus"), 32);
        let files = get_files(&conn);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "my_1337_virus 32 bytes");
    }

    #[test]
    fn gets_all_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("file1"), 32);
        add_file(&conn, &String::from("file2"), 64);
        add_file(&conn, &String::from("file3"), 128);
        let files = get_files(&conn);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1 32 bytes");
        assert_eq!(files[1], "file2 64 bytes");
        assert_eq!(files[2], "file3 128 bytes");
    }

    #[test]
    fn returns_empty_list_if_no_files_exist() {
        let conn = get_test_db();
        let files = get_files(&conn);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn adds_blocks_to_inode() {
        let conn = get_test_db();
        let filename = String::from("main_file");
        add_file(&conn, &filename, 128);
        add_data_node(&conn, "127.0.0.1", 1337);
        add_data_node(&conn, "127.0.0.2", 1338);
        let inode = get_file_info(&conn, &filename);
        let blocks = vec!(
            Block {
                file_id: inode.id,
                id: 0,
                node_id: 1,
                chunk_id: String::from("c1"),
            },
            Block {
                file_id: inode.id,
                id: 0,
                node_id: 2,
                chunk_id: String::from("c2"),
            },
        );
        add_blocks_to_inode(&conn, &filename, &blocks);
        let (inode, blocks) = get_file_inode(&conn, &filename);
        assert_eq!(inode.name, "main_file");
        assert_eq!(inode.size, 128);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].chunk_id, "c1");
        assert_eq!(blocks[0].data_node.id, 1);
        assert_eq!(blocks[0].data_node.ip, "127.0.0.1");
        assert_eq!(blocks[0].data_node.port, 1337);
        assert_eq!(blocks[1].chunk_id, "c2");
        assert_eq!(blocks[1].data_node.id, 2);
        assert_eq!(blocks[1].data_node.ip, "127.0.0.2");
        assert_eq!(blocks[1].data_node.port, 1338);
    }

}
