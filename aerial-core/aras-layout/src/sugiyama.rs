use crate::graph::LayoutGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

pub fn layout(graph: &mut LayoutGraph) {
    if graph.node_count() == 0 {
        return;
    }

    // 1. Layer Assignment (Longest Path via Kahn's Algorithm)
    let mut in_degrees = HashMap::new();
    for idx in graph.node_indices() {
        in_degrees.insert(idx, 0);
    }
    for edge in graph.raw_edges() {
        *in_degrees.entry(edge.target()).or_insert(0) += 1;
    }

    let mut queue: Vec<NodeIndex> = in_degrees.iter()
        .filter(|&(_, &deg)| deg == 0)
        .map(|(&idx, _)| idx)
        .collect();

    let mut layers: HashMap<NodeIndex, usize> = HashMap::new();
    for &q in &queue {
        layers.insert(q, 0);
    }

    let mut max_layer = 0;
    while let Some(u) = queue.pop() {
        let u_layer = layers[&u];
        for v in graph.neighbors(u) {
            let current_v_layer = layers.get(&v).copied().unwrap_or(0);
            layers.insert(v, current_v_layer.max(u_layer + 1));
            max_layer = max_layer.max(u_layer + 1);

            let deg = in_degrees.get_mut(&v).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push(v);
            }
        }
    }

    // Cycle fallback: any nodes in a cycle (in-degree never reached 0) get layered at the bottom
    for idx in graph.node_indices() {
        if !layers.contains_key(&idx) {
            layers.insert(idx, max_layer + 1);
        }
    }
    max_layer += 1;

    // Group nodes by layer
    let mut layer_nodes: Vec<Vec<NodeIndex>> = vec![Vec::new(); max_layer + 1];
    for (idx, &layer) in &layers {
        layer_nodes[layer].push(*idx);
        graph[*idx].layer = layer;
    }

    // 2. Coordinate Assignment
    let node_spacing_x = 160.0;
    let node_spacing_y = 160.0;
    
    let mut max_width_per_layer = vec![0.0; max_layer + 1];
    for l in 0..=max_layer {
        let mut total_width = 0.0;
        for &node in &layer_nodes[l] {
            total_width += graph[node].width + node_spacing_x;
        }
        max_width_per_layer[l] = total_width;
    }

    let max_total_width = max_width_per_layer.iter().cloned().fold(0.0, f64::max);

    // Assign X and Y
    for l in 0..=max_layer {
        let y = l as f64 * node_spacing_y + 80.0;
        
        // Sort nodes in this layer by group name so groups are clustered together!
        layer_nodes[l].sort_by_key(|&n| graph[n].group.clone().unwrap_or_default());
        
        // Recompute max_width_per_layer since we might want to add extra spacing between DIFFERENT groups
        let mut x = 80.0;
        let mut last_group: Option<String> = None;
        
        for &node in &layer_nodes[l] {
            let current_group = graph[node].group.clone();
            if last_group.is_some() && current_group != last_group {
                x += 80.0; // Extra gap between different groups
            }
            
            graph[node].x = x;
            graph[node].y = y;
            x += graph[node].width + node_spacing_x;
            last_group = current_group;
        }
        max_width_per_layer[l] = x;
    }

    let max_total_width = max_width_per_layer.iter().cloned().fold(0.0, f64::max);
    
    // Center each layer
    for l in 0..=max_layer {
        let offset = (max_total_width - max_width_per_layer[l]) / 2.0;
        for &node in &layer_nodes[l] {
            graph[node].x += offset;
        }
    }

    // 3. Edge Routing
    for edge_idx in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge_idx).unwrap();
        
        let sx = graph[source].x + graph[source].width / 2.0;
        let sy = graph[source].y + graph[source].height;
        
        let tx = graph[target].x + graph[target].width / 2.0;
        let ty = graph[target].y;

        let edge = graph.edge_weight_mut(edge_idx).unwrap();
        edge.points = vec![(sx, sy), (tx, ty)];
    }
}
