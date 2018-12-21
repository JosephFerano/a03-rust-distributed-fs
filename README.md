## Distributed File System in Rust for CCOM4017

This suite of programs handles file copying over TCP with a client/server model.
It contains the following programs;
- copy
- ls
- data_node
- meta_data 

`copy` and `ls` are clients that connect to the servers. `copy` sends file read and write requests
to the `meta_data` server, which uses a sqlite3 database to keep track of which nodes are connected,
as well as which files have been added. When a file is added, `meta_data` sends the list of available
`data_node` servers, `copy` then divides the file up by the amount of nodes, then proceeds to transfer
each chunk over 256 bytes at a time. `ls` simply prints out a list of the existing files on the
`meta_data` server.

The code uses `serde_json` to serialize and deserialize Rust structs to and from json. The clients and
servers then listen for incoming streams of data and parses them as json. As well as exchanging
metadata, this protocol also establishes the handshake to then transfer the raw file chunks.

`rusqlite` is used for managing the sqlite database. This allows SQL queries to be performed from
the rust code and manage the data base in a relatively type safe way. Unit tests in the `meta_data`
provide coverage of these SQL operations against an in-memory version

### WARNING: 
If you're my professor, please do not generate a database with the default `createdb.py` 
provided in the skeleton dfs. I have included a custom version of the file in the root of the project.
The reason being that I changed chunks to be integers rather than strings, in order to provide ordering
to the chunks when transferring.

##### Running

To run the `ls` provide an endpoint in the _`ip:port`_ format. _`ip`_ can be _"localhost"_, consider 
using `./` to avoid a naming conflict with the GNU version of `ls`

```$ ./ls 127.0.0.1:6770```

The `meta_data` server takes an optional port, but will default to `8000` if none is specified.

```$ meta_data 6710```

The data node takes two endpoints in the _`ip:port`_ and then a an optional path. The first endpoint
is the ip and port, both for binding to a TCP port and also to send itself to the `meta_data` server.
The second endpoint is the `meta_data` server's ip and port. The optional base path will default to the
working directory if none is provided. 

```$ data_node localhost:6771 127.0.0.1:8000 my_cool_data_node```

The `copy` takes two different parameter versions, depending on whether it's sending to or receiving 
from the server. To send a file, provide the path to the local file, then the endpoint with the file
in the _`ip:host:filepath`_ format. The `data_node` will save the file relative to the base path
provided to it. 

```$ copy some_path/pug.jpg localhost:6700:another_path/pug.jpg```

To receive a file, simply invert the parameters

```$ copy localhost:6700:another_path/pug.jpg some_path/pug.jpg```

##### Misc Scripts

`shutdown_node` sends a json request with a provided port to shutdown a `data_node`. This ensures
that the node can terminate gracefully and unregister itself from the `meta_data` server. I was
advised against using Unix Signals, so opted for this instead.

```$ shutdown_node 6770```

`sm` just does a _send message_ to a provide port. It can be used to test and inspect jsons. It can
for instance be used to mimic the `ls`;

```
$ sm '{"p_type":"ListFiles","json":null}' 8000
Connection to localhost 8000 port [tcp/*] succeeded!
{"paths":["pug.jpg 21633 bytes"]}%
```

`clean_db` just recreates the `dfs.db` with the custom python script.

##### Building

If you wish to compile the code, install rust and cargo
[link](https://www.rust-lang.org/en-US/install.html)

Then just run build

```cargo build```

If you wish to run a specific algorithm;

```cargo run --bin copy ```

##### Testing

`cargo test --bin meta_data`
