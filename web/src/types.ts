export interface RoamNode {
  title: string;
  id: string;
  parent: string;
  num_links: number;
}

export interface RoamLink {
  from: string;
  to: string;
}

export default interface GraphData {
  nodes: RoamNode[];
  links: RoamLink[];
}

export interface SearchResponse {
  providers: {
    source: string;
    results: {
      display: string;
      id: string;
      tags: string[];
    }[];
  }[];
}

export interface OrgAsHTMLResponse {
  org: string;
  links: {
    display: string;
    id: string;
  }[];
  latex_blocks: string[];
}

export interface WebSocketMessage {
  type: string;
}

export interface StatusUpdateMessage extends WebSocketMessage {
  type: "status_update";
  visited_node: string | null;
  pending_changes: boolean;
  updated_nodes: RoamNode[];
  updated_links: RoamLink[];
}

export interface NodeVisitedMessage extends WebSocketMessage {
  type: "node_visited";
  node_id: string;
}

export interface GraphUpdateMessage extends WebSocketMessage {
  type: "graph_update";
  new_nodes: RoamNode[];
  updated_nodes: RoamNode[];
  new_links: RoamLink[];
  removed_nodes: string[];
  removed_links: RoamLink[];
}

export interface PingMessage extends WebSocketMessage {
  type: "ping";
}

export interface PongMessage extends WebSocketMessage {
  type: "pong";
}

export interface SearchRequestMessage extends WebSocketMessage {
  type: "search_request";
  query: string;
  request_id: string;
}

export interface SearchResponseMessage extends WebSocketMessage {
  type: "search_response";
  request_id: string;
  results: {
    display: string;
    id: string;
    tags: string[];
  }[];
}
