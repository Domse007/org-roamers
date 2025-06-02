<script setup lang="ts">
import { MultiGraph } from "graphology";
import louvain from "graphology-communities-louvain";
import forceAtlas2 from "graphology-layout-forceatlas2";
import FA2Layout from "graphology-layout-forceatlas2/worker";
import Sigma from "sigma";
import { NodeBorderProgram } from "@sigma/node-border";
import type GraphData from "@/types";
import { onMounted, watch } from "vue";
import { generalSettings } from "../settings.ts";
import drawHover from "../sigma_helpers.ts";
import type { RoamLink, RoamNode } from "@/types";

const randomNumber = (min: number, max: number) =>
  Math.random() * (max - min) + min;
let graph = new MultiGraph();
let sigma: Sigma | null = null;

const updateGraph = () => {
  const style = window.getComputedStyle(document.body);
  const nodeColor = style.getPropertyValue("--node");
  const edgeColor = style.getPropertyValue("--overlay");
  const nodeBorderColor = style.getPropertyValue("--node-border");

  fetch(`/graph`)
    .then((resp) => resp.json())
    .then((text) => JSON.parse(text))
    .then((json: GraphData) => {
      json.nodes.forEach(
        (node: {
          title: string;
          id: string;
          parent: string;
          num_links: number;
        }) => {
          graph.addNode(node.id, {
            label: node.title,
            x: randomNumber(1, 100),
            y: randomNumber(1, 100),
            size: node.num_links / 2 <= 5 ? 5 : node.num_links / 2,
            color: nodeColor,
            borderColor: nodeBorderColor,
          });
        },
      );
      json.links.forEach((edge: { from: string; to: string }) => {
        try {
          graph.addEdge(edge.from, edge.to, { color: edgeColor });
        } catch (error) {
          console.log(`${edge.from}->${edge.to}: ${error}`);
        }
      });
      let count = 0;
      // iterate again to get all parent links
      json.nodes.forEach(
        (node: { title: string; id: string; parent: string }) => {
          if (node.parent != null && node.parent.length != 0) {
            try {
              count++;
              // probably broken, because it's not the id of self.
              graph.addEdge(node.id, node.parent, { color: edgeColor });
            } catch (error) {
              console.log(`ERROR :: ${node.id} -> ${node.parent}: ${error}`);
            }
          }
        },
      );
      console.log(`Counted ${count} olp links.`);
      setupGraph();
    });
};

let layout: FA2Layout;

function setupGraph() {
  const settings = forceAtlas2.inferSettings(graph);
  layout = new FA2Layout(graph, {
    settings: settings,
  });

  // Graph coloring
  const c = (v: string) =>
    getComputedStyle(document.documentElement).getPropertyValue(v).trim();
  const colors = [
    c("--overlay"),
    c("--highlight"),
    c("--highlight-2"),
    c("--warn"),
    c("--clickable"),
    c("--node"),
    c("--node-border"),
    c("--keyword"),
    c("--ident"),
    c("--comment"),
    c("--type"),
  ];
  const communities = louvain(graph);
  Object.entries(communities).forEach(([node, communityId]) => {
    const color = colors[communityId % colors.length];
    graph.mergeNodeAttributes(node, {
      community: communityId,
      color,
    });
  });

  // Graph layouting
  layout.start();
  const element = document.getElementById("graph")!;
  const style = window.getComputedStyle(document.body);
  const textColor = style.getPropertyValue("--text");
  console.log(`text color: ${textColor}`);
  sigma = new Sigma(graph, element, {
    defaultNodeType: "bordered",
    nodeProgramClasses: {
      bordered: NodeBorderProgram,
    },
    defaultDrawNodeHover: drawHover,
    labelColor: { color: textColor },
  });

  sigma.on("downNode", (e) => {
    zoomOnto(e.node, old_node);
    emit("openNode", e.node);
  });

  if (generalSettings.stopLayoutAfter != null) {
    setTimeout(() => layout.stop(), generalSettings.stopLayoutAfter * 1000);
  }
}

const zoomOnto = (id: string, old_id: string) => {
  const c = (v: string) =>
    getComputedStyle(document.documentElement).getPropertyValue(v).trim();
  // reset old highlighted node edges
  if (old_id.length != 0) highlightEdgesFromNode(old_id, c("--overlay"));

  const ratio = 0.2;
  // NOTE: 0.4 was chosen because it worked on my machine. Don't know about others.
  const graphOffset = 0.4 * ratio;
  const position = sigma!.getNodeDisplayData(id)!;

  const actualZoom = () => {
    sigma!.getCamera().animate(
      {
        x: position.x + graphOffset,
        y: position.y,
        ratio: ratio,
      },
      {
        duration: 1000,
      },
    );
    highlightEdgesFromNode(id, c("--highlight-2"));
  };

  try {
    actualZoom();
  } catch (_) {}
};

function highlightEdgesFromNode(sourceNode: unknown, color: string) {
  graph.forEachEdge(sourceNode, (edge) => {
    graph.setEdgeAttribute(edge, "color", color);
  });
}

const incrementalGraphUpdate = (updates: {
  nodes: RoamNode[];
  links: RoamLink[];
}) => {
  const style = window.getComputedStyle(document.body);
  const nodeColor = style.getPropertyValue("--node");
  const nodeBorderColor = style.getPropertyValue("--node-border");
  for (const node of updates.nodes) {
    try {
      graph.addNode(node.id, {
        label: node.title,
        x: randomNumber(1, 100),
        y: randomNumber(1, 100),
        size: node.num_links / 2 <= 5 ? 5 : node.num_links / 2,
        color: nodeColor,
        borderColor: nodeBorderColor,
      });
    } catch (err) {
      console.log(`${node.id} (${node.title}): ${err}`);
    }
  }
  const edgeColor = style.getPropertyValue("--overlay");
  for (const link of updates.links) {
    try {
      graph.addEdge(link.from, link.to, { color: edgeColor });
    } catch (error) {
      console.log(`${link.from}->${link.to}: ${error}`);
    }
  }
  document.getElementById("graph")!.innerHTML = "";
  setupGraph();
  // bit sketchy...
  zoomOnto(old_node, old_node);
};

const prop = defineProps<{
  count: number;
  toggleLayouter: boolean;
  zoomNode: string;
  updates: { nodes: RoamNode[]; links: RoamLink[] } | null;
}>();

let old_count = 0;
let old_layout_state = false;
let old_node = "";

watch(prop, () => {
  if (old_count != prop.count) {
    console.log("Trying to update");
    graph = new MultiGraph();
    sigma = null;
    document.getElementById("graph")!.innerHTML = "";
    updateGraph();
    old_count = prop.count;
  }
  if (old_layout_state != prop.toggleLayouter) {
    if (layout.isRunning()) layout.stop();
    else layout.start();
    old_layout_state = prop.toggleLayouter;
  }
  if (old_node != prop.zoomNode) {
    zoomOnto(prop.zoomNode, old_node);
    old_node = prop.zoomNode;
  }
  if (prop.updates != null) {
    console.log("STARTING TO UPDATE GRAPH");
    incrementalGraphUpdate(prop.updates);
  }
});

onMounted(updateGraph);
const emit = defineEmits(["openNode"]);
</script>

<template>
  <div id="graph"></div>
</template>

<style scoped>
#graph {
  top: 0px;
  left: 0px;
  bottom: 0px;
  right: 0px;
  width: 100%;
  height: 100%;
  position: absolute;
  background-color: var(--base);
}
</style>
