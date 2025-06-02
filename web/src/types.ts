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

export interface ServerStatus {
  visited_node: string | null;
  pending_changes: boolean;
  updated_nodes: RoamNode[];
  updated_links: RoamLink[];
}

export interface OrgAsHTMLResponse {
  org: string;
  links: {
    display: string;
    id: string;
  }[];
}
