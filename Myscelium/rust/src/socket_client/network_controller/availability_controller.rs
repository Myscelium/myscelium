use std::collections::HashMap;

pub enum NodeStatus {
    Online,
    Idle,
    NotSync,
    Offline,
}

pub struct RemoteNode {
    node_key: String,
    node_status: NodeStatus,
    remote_handlers_allowed: HashMap<String, serde_json::Value>,
}

// TODO >>> Add a way to remember nodes expected, when client restarts for exemple to remember things that was removed and be able to see if the dependence expected was removed

pub struct AllowedNetWorkController {
    remote_nodes: Vec<RemoteNode>,
}

impl RemoteNode {
    pub fn new(node_key: String, node_status: NodeStatus, remote_handlers_allowed: HashMap<String, serde_json::Value>) -> Self {
        Self {
            node_key,
            node_status,
            remote_handlers_allowed,
        }
    }
}

pub enum NetworkControllerError {
    RemoteHandlerDoesntExists,
    TargetIsOffline,
    TargetDoesntExists,
    TargetIsntSyncYet,
    InvalidPattern(String),
}

impl AllowedNetWorkController {
    pub fn new() -> Self {
        Self { remote_nodes: Vec::new() }
    }

    pub fn add_node(&mut self, remote_node: RemoteNode) {
        self.remote_nodes.push(remote_node);
    }

    pub fn update_from_pattern(&mut self, node_pattern: Vec<HashMap<String, serde_json::Value>>) -> Result<(), NetworkControllerError> {
        for new_node in node_pattern {
            self.update_node_from_pattern(&new_node)?;
        }
        Ok(())
    }

    fn update_node_from_pattern(&mut self, new_node: &HashMap<String, serde_json::Value>) -> Result<(), NetworkControllerError> {
        let new_node_key = new_node.get("node_key").and_then(|v| v.as_str()).ok_or(NetworkControllerError::InvalidPattern("New node missing key!".to_string()))?;

        // > Update Node Status Helper
        fn update_node_status(node: &mut RemoteNode, new_node: &HashMap<String, serde_json::Value>) -> Result<(), NetworkControllerError> {
            let new_node_status = new_node.get("node_status").and_then(|v| v.as_str()).ok_or(NetworkControllerError::InvalidPattern("New node missing status!".to_string()))?;

            node.node_status = match new_node_status {
                "Online" => NodeStatus::Online,
                "Idle" => NodeStatus::Idle,
                "NotSync" => NodeStatus::NotSync,
                "Offline" => NodeStatus::Offline,
                _ => return Err(NetworkControllerError::InvalidPattern("New node has an invalid status!".to_string())),
            };
            Ok(())
        }

        // > Update Node Helper
        fn update_node_handlers(node: &mut RemoteNode, new_node: &HashMap<String, serde_json::Value>) -> Result<(), NetworkControllerError> {
            let node_handlers = new_node.get("node_handlers").and_then(|v| v.as_object()).ok_or(NetworkControllerError::InvalidPattern("New node missing handlers!".to_string()))?;

            for (handler_name, handler_value) in node_handlers {
                node.remote_handlers_allowed.insert(handler_name.clone(), handler_value.clone());
            }
            Ok(())
        }

        // > UPDATE NODES
        for node in &mut self.remote_nodes {
            if &node.node_key != new_node_key {
                continue;
            }

            update_node_status(node, new_node)?;
            update_node_handlers(node, new_node)?;
            return Ok(());
        }
        Ok(())
    }

    pub fn remote_handler_is_reachable(&self, target_key: &String, remote_handler_expected: &String) -> Result<(), NetworkControllerError> {
        for c in &self.remote_nodes {
            if &c.node_key == target_key {
                match c.node_status {
                    NodeStatus::Online => {
                        //> Pass
                    },
                    NodeStatus::NotSync => {
                        return Err(NetworkControllerError::TargetIsntSyncYet);
                    },
                    NodeStatus::Idle => {
                        //* Pass Idle doen't mean offline, it can turn online in midle of the percurse, and will be in host buffer until processed
                    },
                    NodeStatus::Offline => {
                        return Err(NetworkControllerError::TargetIsOffline);
                    },
                }
                if c.remote_handlers_allowed.contains_key(remote_handler_expected) {
                    return Ok(());
                } else {
                    return Err(NetworkControllerError::RemoteHandlerDoesntExists);
                }
            } else {
                continue;
            }
        }

        return Err(NetworkControllerError::TargetDoesntExists);
    }
}
