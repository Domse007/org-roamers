export default interface GraphData {
  nodes: {
    title: string;
    id: string;
    parent: string;
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
    }[];
  }[];
}
