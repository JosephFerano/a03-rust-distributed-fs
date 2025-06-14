# Distributed File System in Rust

A distributed file system implementation with client-server architecture, built as one of my early projects exploring Rust and distributed systems concepts.

## Architecture

The system consists of four main components:

- **copy** - Client for reading and writing files to the distributed system
- **ls** - Client for listing files stored in the system
- **data_node** - Storage server that handles file chunks
- **meta_data** - Metadata server managing file locations and node registry

## How It Works

The metadata server uses SQLite to track connected data nodes and file locations. When storing a file, the client connects to the metadata server, which provides a list of available data nodes. The client then divides the file into chunks and distributes them across multiple data nodes, transferring 256 bytes at a time.

The system uses JSON serialization via `serde_json` for communication between components. All network communication happens over TCP with a custom protocol for coordinating file operations.

## Usage

### Starting the Metadata Server
```bash
cargo run --bin meta_data [port]
# Defaults to port 8000 if not specified
```

### Starting Data Nodes
```bash
cargo run --bin data_node <node_endpoint> <metadata_endpoint> [base_path]
# Example: cargo run --bin data_node localhost:6771 127.0.0.1:8000 ./data
```

### Listing Files
```bash
cargo run --bin ls <metadata_endpoint>
# Example: cargo run --bin ls 127.0.0.1:8000
```

### Copying Files

**Upload to distributed system:**
```bash
cargo run --bin copy <local_file> <endpoint:remote_path>
# Example: cargo run --bin copy ./document.pdf localhost:8000:docs/document.pdf
```

**Download from distributed system:**
```bash
cargo run --bin copy <endpoint:remote_path> <local_file>
# Example: cargo run --bin copy localhost:8000:docs/document.pdf ./document.pdf
```

## Database Setup

Run the included Python script to initialize the SQLite database:
```bash
python3 createdb.py
```

The database schema includes tables for file metadata (inodes), data node registry, and block location tracking.

## Building

```bash
cargo build
```

## Testing

```bash
cargo test --bin meta_data
```

## Dependencies

- **rusqlite** - SQLite database interface
- **serde** - Serialization framework
- **serde_json** - JSON support for network protocol