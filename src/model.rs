pub mod model{
    use std::collections::{HashMap, HashSet};

use uuid::Uuid;


    pub struct Pipe{
        pub tx: Vec<String>,
        pub rx: Vec<String>,
        pub is_same: bool
    }

    pub enum e_Pipe_io{ 
        Rx,
        Tx        
    }

    impl Pipe{
        pub fn new(is_same: bool) -> Self{
            Self { tx: vec![], rx: vec![], is_same}
        }

        pub fn append(&mut self,data: String,ep: Option<e_Pipe_io>) {
            if self.is_same{
                self.tx.push(data);
            }else{
                if let Some(e) = ep{
                    match e{
                        e_Pipe_io::Rx => {
                            self.rx.push(data);
                        },
                        e_Pipe_io::Tx => {
                            self.tx.push(data);
                        },
                    }
                }else{
                    self.tx.push(data.clone());
                    self.rx.push(data);
                }
            }
        }

    pub fn peek(&self,ep:Option<e_Pipe_io>) -> (Option<String>,Option<String>){
        if let Some(e) = ep{
            match e{
                e_Pipe_io::Rx => {
                    return (self.rx.last().cloned(),None);
                },
                e_Pipe_io::Tx => {
                    return (None,self.tx.last().cloned());
                }
            }
        }else{
            return (self.rx.last().cloned(),self.tx.last().cloned());
        }
        
    }

    pub fn history(&self,ep:Option<e_Pipe_io>) -> (Option<String>,Option<String>){
        if let Some(e) = ep{
            match e{
                e_Pipe_io::Rx => {
                    return (Some(self.rx.clone().join("\n\n")),None);
                },
                e_Pipe_io::Tx => {
                    return (None,Some(self.tx.clone().join("\n\n")));
                }
            }
        }else{
            return (Some(self.rx.clone().join("\n\n")),Some(self.tx.clone().join("\n\n")))
        }        
    }

    pub fn len(&self,ep: e_Pipe_io) -> usize{
        match ep{
            e_Pipe_io::Rx => self.rx.len(),
            e_Pipe_io::Tx => self.tx.len(),
        }
    }

    pub fn pop(&self,ep:Option<e_Pipe_io>) -> (Option<String>,Option<String>){
        if let Some(e) = ep{
            match e{
                e_Pipe_io::Rx => {
                    return (self.rx.clone().pop(),None);
                },
                e_Pipe_io::Tx => {
                    return (None,self.tx.clone().pop());
                }
            }
        }else{
            return (self.rx.clone().pop(),self.tx.clone().pop());
        }        
    }

    }



    #[derive(Debug,Clone)]
    pub struct KGraph<'a>{
        pub tree: Option<KNode<'a>>,
        pub adj_l: HashMap<uuid::Uuid,HashMap<uuid::Uuid,KNode<'a>>>
    }

    impl <'a> KGraph<'a>{
        pub fn new() -> Self{
            let root = KNode::new(b"ROOT");
            let root_uuid = root.uuid;
            let mut adj_l = HashMap::new();
            adj_l.insert(root_uuid, HashMap::new());
            Self{tree:Some(root),adj_l}
        }

        pub fn add_node(&mut self, data: &'a [u8]) -> Uuid {
            let node = KNode::new(data);
            let uuid = node.uuid;
            self.adj_l.insert(uuid, HashMap::new());
            uuid
        }

        pub fn add_edge(&mut self, parent_uuid: Uuid, child_uuid: Uuid) -> bool {
            if !self.adj_l.contains_key(&parent_uuid) || !self.adj_l.contains_key(&child_uuid) {
                return false;
            }

            self.adj_l.get_mut(&parent_uuid).unwrap().insert(child_uuid, KNode::new(b""));
            true
        }

        pub fn remove_node(&mut self, uuid: Uuid) -> bool {
            if !self.adj_l.contains_key(&uuid) {
                return false;
            }

            self.adj_l.remove(&uuid);
            for neighbors in self.adj_l.values_mut() {
                neighbors.remove(&uuid);
            }
            true
        }

        pub fn remove_edge(&mut self, parent_uuid: Uuid, child_uuid: Uuid) -> bool {
            if let Some(neighbors) = self.adj_l.get_mut(&parent_uuid) {
                neighbors.remove(&child_uuid);
                return true;
            }
            false
        }

        pub fn get_node(&self, uuid: Uuid) -> Option<&KNode<'a>> {
            self.adj_l.get(&uuid).map(|_| {
                for node in self.adj_l.values() {
                    for n in node.values() {
                        if n.uuid == uuid {
                            return n;
                        }
                    }
                }
                if let Some(root) = &self.tree {
                    if root.uuid == uuid {
                        return root;
                    }
                }
                unreachable!()
            })
        }

        pub fn get_neighbors(&self, uuid: Uuid) -> Vec<Uuid> {
            self.adj_l.get(&uuid)
                .map(|neighbors| neighbors.keys().cloned().collect())
                .unwrap_or_default()
        }

        pub fn has_cycle(&self) -> bool {
            let mut visited = HashSet::new();
            let mut recursion_stack = HashSet::new();

            for uuid in self.adj_l.keys() {
                if !visited.contains(uuid) {
                    if self.dfs_cycle_check(*uuid, &mut visited, &mut recursion_stack) {
                        return true;
                    }
                }
            }
            false
        }

        fn dfs_cycle_check(&self, uuid: Uuid, visited: &mut HashSet<Uuid>, recursion_stack: &mut HashSet<Uuid>) -> bool {
            visited.insert(uuid);
            recursion_stack.insert(uuid);

            if let Some(neighbors) = self.adj_l.get(&uuid) {
                for neighbor_uuid in neighbors.keys() {
                    if !visited.contains(neighbor_uuid) {
                        if self.dfs_cycle_check(*neighbor_uuid, visited, recursion_stack) {
                            return true;
                        }
                    } else if recursion_stack.contains(neighbor_uuid) {
                        return true;
                    }
                }
            }

            recursion_stack.remove(&uuid);
            false
        }

        pub fn get_next_hops(&self, uuid: Uuid) -> Vec<Uuid> {
            self.get_neighbors(uuid)
        }

        pub fn get_next_hops_cycle_aware(&self, uuid: Uuid, visited: &HashSet<Uuid>) -> Vec<Uuid> {
            self.get_neighbors(uuid)
                .into_iter()
                .filter(|neighbor_uuid| !visited.contains(neighbor_uuid))
                .collect()
        }

        pub fn dfs_traverse(&self, start_uuid: Uuid) -> Vec<Uuid> {
            let mut visited = HashSet::new();
            let mut result = Vec::new();
            self.dfs_helper(start_uuid, &mut visited, &mut result);
            result
        }

        fn dfs_helper(&self, uuid: Uuid, visited: &mut HashSet<Uuid>, result: &mut Vec<Uuid>) {
            visited.insert(uuid);
            result.push(uuid);

            for neighbor_uuid in self.get_neighbors(uuid) {
                if !visited.contains(&neighbor_uuid) {
                    self.dfs_helper(neighbor_uuid, visited, result);
                }
            }
        }

        pub fn bfs_traverse(&self, start_uuid: Uuid) -> Vec<Uuid> {
            let mut visited = HashSet::new();
            let mut result = Vec::new();
            let mut queue = std::collections::VecDeque::new();

            visited.insert(start_uuid);
            queue.push_back(start_uuid);

            while let Some(uuid) = queue.pop_front() {
                result.push(uuid);

                for neighbor_uuid in self.get_neighbors(uuid) {
                    if !visited.contains(&neighbor_uuid) {
                        visited.insert(neighbor_uuid);
                        queue.push_back(neighbor_uuid);
                    }
                }
            }

            result
        }

        pub fn cycle_aware_traverse(&self, start_uuid: Uuid) -> Vec<Uuid> {
            let mut visited = HashSet::new();
            let mut result = Vec::new();
            self.cycle_aware_dfs(start_uuid, &mut visited, &mut result);
            result
        }

        fn cycle_aware_dfs(&self, uuid: Uuid, visited: &mut HashSet<Uuid>, result: &mut Vec<Uuid>) {
            visited.insert(uuid);
            result.push(uuid);

            for neighbor_uuid in self.get_next_hops_cycle_aware(uuid, visited) {
                self.cycle_aware_dfs(neighbor_uuid, visited, result);
            }
        }

        pub fn find_path(&self, start_uuid: Uuid, end_uuid: Uuid) -> Option<Vec<Uuid>> {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            
            if self.dfs_path(start_uuid, end_uuid, &mut visited, &mut path) {
                Some(path)
            } else {
                None
            }
        }

        fn dfs_path(&self, current: Uuid, target: Uuid, visited: &mut HashSet<Uuid>, path: &mut Vec<Uuid>) -> bool {
            visited.insert(current);
            path.push(current);

            if current == target {
                return true;
            }

            for neighbor_uuid in self.get_neighbors(current) {
                if !visited.contains(&neighbor_uuid) {
                    if self.dfs_path(neighbor_uuid, target, visited, path) {
                        return true;
                    }
                }
            }

            path.pop();
            false
        }

        pub fn find_all_paths(&self, start_uuid: Uuid, end_uuid: Uuid) -> Vec<Vec<Uuid>> {
            let mut visited = HashSet::new();
            let mut all_paths = Vec::new();
            let mut current_path = Vec::new();
            
            self.dfs_all_paths(start_uuid, end_uuid, &mut visited, &mut current_path, &mut all_paths);
            all_paths
        }

        fn dfs_all_paths(&self, current: Uuid, target: Uuid, visited: &mut HashSet<Uuid>, current_path: &mut Vec<Uuid>, all_paths: &mut Vec<Vec<Uuid>>) {
            visited.insert(current);
            current_path.push(current);

            if current == target {
                all_paths.push(current_path.clone());
            } else {
                for neighbor_uuid in self.get_neighbors(current) {
                    if !visited.contains(&neighbor_uuid) {
                        self.dfs_all_paths(neighbor_uuid, target, visited, current_path, all_paths);
                    }
                }
            }

            current_path.pop();
            visited.remove(&current);
        }

        pub fn topological_sort(&self) -> Option<Vec<Uuid>> {
            if self.has_cycle() {
                return None;
            }
            let mut in_degree = HashMap::new();
            let mut result = Vec::new();
            let mut queue = std::collections::VecDeque::new();
            for uuid in self.adj_l.keys() {
                in_degree.insert(*uuid, 0);
            }

            for neighbors in self.adj_l.values() {
                for neighbor_uuid in neighbors.keys() {
                    *in_degree.entry(*neighbor_uuid).or_insert(0) += 1;
                }
            }

            for (uuid, degree) in &in_degree {
                if *degree == 0 {
                    queue.push_back(*uuid);
                }
            }

            while let Some(uuid) = queue.pop_front() {
                result.push(uuid);

                for neighbor_uuid in self.get_neighbors(uuid) {
                    let degree = in_degree.get_mut(&neighbor_uuid).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor_uuid);
                    }
                }
            }

            if result.len() == self.adj_l.len() {
                Some(result)
            } else {
                None
            }
        }

        pub fn find_nodes_within_distance(&self, start_uuid: Uuid, min_dist: usize, max_dist: usize) -> Vec<(Uuid, usize)> {
            let mut result = Vec::new();
            let mut queue = std::collections::VecDeque::new();
            let mut visited = HashSet::new();

            queue.push_back((start_uuid, 0));
            visited.insert(start_uuid);

            while let Some((uuid, dist)) = queue.pop_front() {
                if dist >= min_dist && dist <= max_dist && uuid != start_uuid {
                    result.push((uuid, dist));
                }

                if dist < max_dist {
                    for neighbor_uuid in self.get_neighbors(uuid) {
                        if !visited.contains(&neighbor_uuid) {
                            visited.insert(neighbor_uuid);
                            queue.push_back((neighbor_uuid, dist + 1));
                        }
                    }
                }
            }

            result.sort_by_key(|(_, dist)| *dist);
            result
        }




    }


    #[derive(Debug,Clone)]
    pub struct KNode<'a>{
        pub data: &'a [u8],
        pub uuid: Uuid,
        pub children: Vec<Uuid>,
        pub parents: Vec<Uuid>,
        pub visited: bool,
    }

    impl <'a> KNode<'a>{
        pub fn new(data: &'a [u8]) -> Self{
            Self { data, uuid: Uuid::new_v4(),children: Vec::new(),parents: Vec::new(),visited: false}
        }

        pub fn add_child(&mut self, child_uuid: Uuid) {
            if !self.children.contains(&child_uuid) {
                self.children.push(child_uuid);
            }
        }

        pub fn add_parent(&mut self, parent_uuid: Uuid) {
            if !self.parents.contains(&parent_uuid) {
                self.parents.push(parent_uuid);
            }
        }

        pub fn remove_child(&mut self, child_uuid: &Uuid) {
            self.children.retain(|x| x != child_uuid);
        }

        pub fn remove_parent(&mut self, parent_uuid: &Uuid) {
            self.parents.retain(|x| x != parent_uuid);
        }

        pub fn get_str(&self) -> String{
            String::from_utf8(self.data.to_vec()).unwrap()
        }
    }


    pub fn generate_sample_graph() -> KGraph<'static> {
        let mut graph = KGraph::new();
        
        // Linear Data Structures
        let array_node = graph.add_node(b"Array");
        let dynamic_array_node = graph.add_node(b"DynamicArray");
        let linked_list_node = graph.add_node(b"LinkedList");
        let singly_linked_node = graph.add_node(b"SinglyLinkedList");
        let doubly_linked_node = graph.add_node(b"DoublyLinkedList");
        let circular_linked_node = graph.add_node(b"CircularLinkedList");
        
        // Stack and Queue
        let stack_node = graph.add_node(b"Stack");
        let queue_node = graph.add_node(b"Queue");
        let deque_node = graph.add_node(b"Deque");
        let priority_queue_node = graph.add_node(b"PriorityQueue");
        
        // Tree Data Structures
        let tree_node = graph.add_node(b"Tree");
        let binary_tree_node = graph.add_node(b"BinaryTree");
        let bst_node = graph.add_node(b"BinarySearchTree");
        let avl_node = graph.add_node(b"AVLTree");
        let red_black_node = graph.add_node(b"RedBlackTree");
        let b_tree_node = graph.add_node(b"BTree");
        let b_plus_tree_node = graph.add_node(b"BPlusTree");
        let trie_node = graph.add_node(b"Trie");
        let prefix_tree_node = graph.add_node(b"PrefixTree");
        let segment_tree_node = graph.add_node(b"SegmentTree");
        let fenwick_tree_node = graph.add_node(b"FenwickTree");
        let heap_node = graph.add_node(b"Heap");
        let min_heap_node = graph.add_node(b"MinHeap");
        let max_heap_node = graph.add_node(b"MaxHeap");
        let binary_heap_node = graph.add_node(b"BinaryHeap");
        
        // Graph Data Structures
        let graph_node = graph.add_node(b"Graph");
        let directed_graph_node = graph.add_node(b"DirectedGraph");
        let undirected_graph_node = graph.add_node(b"UndirectedGraph");
        let weighted_graph_node = graph.add_node(b"WeightedGraph");
        let complete_graph_node = graph.add_node(b"CompleteGraph");
        let bipartite_graph_node = graph.add_node(b"BipartiteGraph");
        
        // Hash-based Structures
        let hash_table_node = graph.add_node(b"HashTable");
        let hash_map_node = graph.add_node(b"HashMap");
        let hash_set_node = graph.add_node(b"HashSet");
        let bloom_filter_node = graph.add_node(b"BloomFilter");
        
        // Graph Algorithms
        let traversal_node = graph.add_node(b"Traversal");
        let dfs_node = graph.add_node(b"DFS");
        let bfs_node = graph.add_node(b"BFS");
        let cycle_node = graph.add_node(b"Cycle");
        let path_node = graph.add_node(b"Path");
        let shortest_path_node = graph.add_node(b"ShortestPath");
        let dijkstra_node = graph.add_node(b"Dijkstra");
        let bellman_ford_node = graph.add_node(b"BellmanFord");
        let floyd_warshall_node = graph.add_node(b"FloydWarshall");
        let a_star_node = graph.add_node(b"AStar");
        let mst_node = graph.add_node(b"MinimumSpanningTree");
        let prim_node = graph.add_node(b"Prim");
        let kruskal_node = graph.add_node(b"Kruskal");
        let topological_node = graph.add_node(b"TopologicalSort");
        let kahn_node = graph.add_node(b"Kahn");
        let dfs_topo_node = graph.add_node(b"DFSTopological");
        
        // Complexity Concepts
        let o1_node = graph.add_node(b"O1");
        let on_node = graph.add_node(b"On");
        let ologn_node = graph.add_node(b"Ologn");
        let onlogn_node = graph.add_node(b"Onlogn");
        let on2_node = graph.add_node(b"On2");
        
        // Usage Patterns
        let random_access_node = graph.add_node(b"RandomAccess");
        let sequential_access_node = graph.add_node(b"SequentialAccess");
        let insertion_node = graph.add_node(b"FastInsertion");
        let deletion_node = graph.add_node(b"FastDeletion");
        let search_node = graph.add_node(b"FastSearch");
        let memory_efficient_node = graph.add_node(b"MemoryEfficient");
        
        // Linear Structure Relationships
        graph.add_edge(array_node, dynamic_array_node);
        graph.add_edge(linked_list_node, singly_linked_node);
        graph.add_edge(linked_list_node, doubly_linked_node);
        graph.add_edge(linked_list_node, circular_linked_node);
        graph.add_edge(dynamic_array_node, stack_node);
        graph.add_edge(dynamic_array_node, queue_node);
        graph.add_edge(dynamic_array_node, deque_node);
        graph.add_edge(heap_node, priority_queue_node);
        
        // Tree Hierarchy
        graph.add_edge(tree_node, binary_tree_node);
        graph.add_edge(binary_tree_node, bst_node);
        graph.add_edge(bst_node, avl_node);
        graph.add_edge(bst_node, red_black_node);
        graph.add_edge(tree_node, b_tree_node);
        graph.add_edge(b_tree_node, b_plus_tree_node);
        graph.add_edge(trie_node, prefix_tree_node);
        graph.add_edge(tree_node, trie_node);
        graph.add_edge(tree_node, segment_tree_node);
        graph.add_edge(tree_node, fenwick_tree_node);
        graph.add_edge(heap_node, binary_heap_node);
        graph.add_edge(binary_heap_node, min_heap_node);
        graph.add_edge(binary_heap_node, max_heap_node);
        
        // Graph Hierarchy
        graph.add_edge(graph_node, directed_graph_node);
        graph.add_edge(graph_node, undirected_graph_node);
        graph.add_edge(graph_node, weighted_graph_node);
        graph.add_edge(graph_node, complete_graph_node);
        graph.add_edge(graph_node, bipartite_graph_node);
        
        // Hash Structure Relationships
        graph.add_edge(hash_table_node, hash_map_node);
        graph.add_edge(hash_table_node, hash_set_node);
        graph.add_edge(hash_table_node, bloom_filter_node);
        
        // Algorithm Relationships
        graph.add_edge(traversal_node, dfs_node);
        graph.add_edge(traversal_node, bfs_node);
        graph.add_edge(graph_node, traversal_node);
        graph.add_edge(graph_node, cycle_node);
        graph.add_edge(graph_node, path_node);
        graph.add_edge(path_node, shortest_path_node);
        graph.add_edge(shortest_path_node, dijkstra_node);
        graph.add_edge(shortest_path_node, bellman_ford_node);
        graph.add_edge(shortest_path_node, floyd_warshall_node);
        graph.add_edge(shortest_path_node, a_star_node);
        graph.add_edge(graph_node, mst_node);
        graph.add_edge(mst_node, prim_node);
        graph.add_edge(mst_node, kruskal_node);
        graph.add_edge(graph_node, topological_node);
        graph.add_edge(topological_node, kahn_node);
        graph.add_edge(topological_node, dfs_topo_node);
        
        // Complexity Ordering (O(1) < O(log n) < O(n) < O(n log n) < O(n²))
        graph.add_edge(o1_node, ologn_node);
        graph.add_edge(ologn_node, on_node);
        graph.add_edge(on_node, onlogn_node);
        graph.add_edge(onlogn_node, on2_node);
        
        // Data Structure to Complexity Mappings
        graph.add_edge(array_node, o1_node);
        graph.add_edge(array_node, random_access_node);
        graph.add_edge(dynamic_array_node, on_node);
        graph.add_edge(linked_list_node, on_node);
        graph.add_edge(linked_list_node, sequential_access_node);
        graph.add_edge(linked_list_node, insertion_node);
        graph.add_edge(linked_list_node, deletion_node);
        graph.add_edge(stack_node, o1_node);
        graph.add_edge(stack_node, insertion_node);
        graph.add_edge(queue_node, o1_node);
        graph.add_edge(queue_node, insertion_node);
        graph.add_edge(hash_table_node, o1_node);
        graph.add_edge(hash_table_node, search_node);
        graph.add_edge(bst_node, ologn_node);
        graph.add_edge(bst_node, search_node);
        graph.add_edge(avl_node, ologn_node);
        graph.add_edge(avl_node, search_node);
        graph.add_edge(red_black_node, ologn_node);
        graph.add_edge(red_black_node, search_node);
        graph.add_edge(heap_node, o1_node);
        graph.add_edge(heap_node, search_node);
        graph.add_edge(trie_node, on_node);
        graph.add_edge(trie_node, search_node);
        graph.add_edge(bloom_filter_node, o1_node);
        graph.add_edge(bloom_filter_node, memory_efficient_node);
        
        // Algorithm Complexity
        graph.add_edge(dijkstra_node, onlogn_node);
        graph.add_edge(bellman_ford_node, on2_node);
        graph.add_edge(floyd_warshall_node, on2_node);
        graph.add_edge(prim_node, onlogn_node);
        graph.add_edge(kruskal_node, onlogn_node);
        graph.add_edge(topological_node, on_node);
        
        // Usage Pattern Relationships
        graph.add_edge(array_node, random_access_node);
        graph.add_edge(linked_list_node, sequential_access_node);
        graph.add_edge(stack_node, insertion_node);
        graph.add_edge(queue_node, insertion_node);
        graph.add_edge(hash_table_node, search_node);
        graph.add_edge(bloom_filter_node, memory_efficient_node);
        
        // Cross-structure relationships
        graph.add_edge(bst_node, heap_node);
        graph.add_edge(priority_queue_node, heap_node);
        graph.add_edge(hash_map_node, search_node);
        graph.add_edge(hash_set_node, search_node);
        
        graph
    }

    pub fn perform(args: Vec<&str>) -> String{
        let mut ret = "".to_string();
        for (i,v) in args.iter().enumerate(){
            if i == 0{
                
            }
        }
        ret
    }



}