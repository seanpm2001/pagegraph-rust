//! Given an adblock network rule, prints out the nodes for resources that match that rule.

use pagegraph::graph::PageGraph;
use pagegraph::types::{EdgeType, NodeType};

pub fn main(graph: &PageGraph, filter_rules: Vec<String>, only_exceptions: bool) {
    let matching_elements = graph.resources_matching_filters(filter_rules, only_exceptions);

    #[derive(serde::Serialize)]
    struct MatchingResource {
        url: String,
        node_id: String,
        request_types: Vec<String>,
        requests: Vec<(usize, String)>,
    }

    let matching_resources = matching_elements.iter().filter(|(_id, matching_element)| {
        matches!(&matching_element.node_type, NodeType::Resource { .. })
    }).map(|(node_id, resource_node)| {
        let requests = graph.incoming_edges(resource_node).filter_map(|edge| if let EdgeType::RequestStart { request_id, .. } = &edge.edge_type {
            Some((*request_id, format!("{}", edge.id)))
        } else {
            None
        }).collect::<Vec<_>>();

        let request_types = graph.resource_request_types(node_id).into_iter().map(|(ty, _)| ty).collect();

        if let NodeType::Resource { url } = &resource_node.node_type {
            MatchingResource {
                url: url.clone(),
                node_id: format!("{}", node_id),
                request_types,
                requests,
            }
        } else { unreachable!() }
    }).collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&matching_resources).unwrap())
}
