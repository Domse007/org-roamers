export default interface GraphData {
  nodes: {
    title: string;
    id: string;
    parent: string;
    num_links: number;
  }[];
  links: {
    from: string;
    to: string;
  }[];
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
  pending_changes: boolean;
}
