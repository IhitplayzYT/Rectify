# Retify

A knowledge graph system built in Rust that uses Unix FIFO pipes for inter-process communication (IPC). Retify provides a graph-based data structure for representing and querying relationships between various concepts, with a focus on computer science data structures, algorithms, and complexity theory.

## Features

- **Knowledge Graph**: A flexible graph data structure for storing and querying relationships between nodes
- **FIFO-based IPC**: Uses Unix named pipes for communication between processes
- **Graph Operations**: Supports various graph algorithms including:
  - DFS and BFS traversal
  - Path finding (single and all paths)
  - Cycle detection
  - Topological sorting
  - Distance-based node queries
- **Sample Knowledge Base**: Pre-populated with computer science concepts including:
  - Data structures (arrays, trees, graphs, hash tables)
  - Algorithms (traversal, shortest path, MST, sorting)
  - Time complexity classes (O(1), O(log n), O(n), O(n log n), O(n²))
  - Usage patterns (random access, sequential access, search optimization)

## Dependencies

- **Rust**: Edition 2024
- **nix** (v0.29): Unix system calls and FIFO creation
- **uuid** (v1.13.0): UUID generation with serde, v4, and v7 features

## Installation

### Prerequisites

- Rust toolchain (2024 edition or compatible)
- Unix-like operating system (Linux, macOS) with FIFO support

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd Retify

# Build the project
cargo build --release

# The binary will be available at target/release/Retify
```

## Usage

### Basic Usage

Run the application with default FIFO paths:

```bash
cargo run --release
```

This will use `/tmp/iopipe` as both the input and output FIFO.

### Command-Line Options

```bash
# Specify custom input pipe
cargo run -- --ipipe=/tmp/input_pipe

# Specify custom output pipe
cargo run -- --opipe=/tmp/output_pipe

# Use separate pipes for input and output
cargo run -- --ipipe=/tmp/input_pipe --opipe=/tmp/output_pipe

# Enable debug mode
cargo run -- -d

# Toggle session mode (default: enabled)
cargo run -- -s

# Show help
cargo run -- -h
```

### Available Commands

Once the application is running, you can send commands through the input pipe:

#### Get Node Data
Retrieve data for one or more nodes by UUID:
```
get_node <uuid1> <uuid2> ...
```

#### Get Neighbors
Retrieve neighbors of one or more nodes:
```
get_neighbour <uuid1> <uuid2> ...
```

#### Prune Query
Find nodes within a specified distance range:
```
prune <max_distance> <uuid>
prune <min_distance> <max_distance> <uuid1> <uuid2> ...
```

#### Direct UUID Query
Query neighbors of a single UUID (shorthand):
```
<uuid>
```

### Example Session

1. **Start the application**:
```bash
# Terminal 1: Start Retify
cargo run --release -- --ipipe=/tmp/retify_in --opipe=/tmp/retify_out

# Terminal 2: Create the FIFOs (if they don't exist)
mkfifo /tmp/retify_in /tmp/retify_out
```

2. **Send queries**:
```bash
# Terminal 3: Send a query
echo "get_node <uuid>" > /tmp/retify_in

# Read the response
cat /tmp/retify_out
```

3. **Multiple queries** (separated by double newlines):
```bash
echo -e "get_node <uuid1>\n\nget_neighbour <uuid2>" > /tmp/retify_in
```

## Architecture

### Components

- **main.rs**: Entry point, FIFO management, and query processing
- **model.rs**: Core data structures and graph algorithms
  - `KGraph`: Main knowledge graph structure
  - `KNode`: Individual graph nodes
  - `Pipe`: FIFO communication abstraction
- **helper.rs**: Command-line argument parsing

### Data Structures

#### KGraph
The main knowledge graph structure with:
- Adjacency list representation
- Support for directed edges
- Various graph algorithms (traversal, path finding, cycle detection)

#### KNode
Individual graph nodes containing:
- UUID-based identification
- Binary data payload
- Parent and child relationships

#### Pipe
Abstraction for FIFO communication:
- Separate RX/TX buffers
- Support for same-pipe or separate-pipe configurations
- Thread-safe operations using Arc<Mutex<>>

## Sample Knowledge Graph

The application includes a pre-populated knowledge graph with relationships between:

- **Linear Data Structures**: Array, DynamicArray, LinkedList (singly, doubly, circular)
- **Stack/Queue**: Stack, Queue, Deque, PriorityQueue
- **Trees**: Tree, BinaryTree, BST, AVL, RedBlack, B-Tree, Trie, SegmentTree, FenwickTree, Heap
- **Graphs**: Graph, Directed, Undirected, Weighted, Complete, Bipartite
- **Hash Structures**: HashTable, HashMap, HashSet, BloomFilter
- **Algorithms**: DFS, BFS, Dijkstra, Bellman-Ford, Floyd-Warshall, A*, Prim, Kruskal
- **Complexity**: O(1), O(log n), O(n), O(n log n), O(n²)
- **Usage Patterns**: RandomAccess, SequentialAccess, FastInsertion, FastDeletion, FastSearch, MemoryEfficient

## Graph Operations

### Traversal
- `dfs_traverse(start_uuid)`: Depth-first traversal
- `bfs_traverse(start_uuid)`: Breadth-first traversal
- `cycle_aware_traverse(start_uuid)`: DFS with cycle detection

### Path Finding
- `find_path(start_uuid, end_uuid)`: Find shortest path
- `find_all_paths(start_uuid, end_uuid)`: Find all possible paths

### Analysis
- `has_cycle()`: Detect cycles in the graph
- `topological_sort()`: Return topological ordering (if acyclic)
- `find_nodes_within_distance(uuid, min_dist, max_dist)`: Find nodes within distance range

### Node Operations
- `add_node(data)`: Add a new node
- `add_edge(parent_uuid, child_uuid)`: Add directed edge
- `remove_node(uuid)`: Remove node and associated edges
- `remove_edge(parent_uuid, child_uuid)`: Remove edge
- `get_node(uuid)`: Retrieve node by UUID
- `get_neighbors(uuid)`: Get adjacent nodes

## License

GPL-3.0-only - See LICENSE file for details

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust naming conventions (snake_case for functions/variables, UpperCamelCase for types)
- All warnings are addressed before submitting PRs
- New features include appropriate tests

## Troubleshooting

### FIFO Creation Issues
If you encounter permission issues with FIFO creation:
```bash
# Ensure /tmp directory is writable
sudo chmod 1777 /tmp

# Or specify a custom pipe location in a writable directory
cargo run -- --ipipe=/path/to/writable/input_pipe
```

### Build Errors
Ensure you have the correct Rust edition:
```bash
rustup update
rustup default stable
```

### UUID Parsing
When querying nodes, ensure UUIDs are in the standard format:
```
550e8400-e29b-41d4-a716-446655440000
```
