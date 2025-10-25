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
  const nodeColor = style.getPropertyValue("--node").trim();
  const edgeColor = style.getPropertyValue("--overlay").trim();
  const nodeBorderColor = style.getPropertyValue("--node-border").trim();

  const params = new URLSearchParams();
  if (prop.filterTags && prop.filterTags.length > 0) {
    params.append("tags", prop.filterTags.join(","));
  }
  if (prop.excludeTags && prop.excludeTags.length > 0) {
    params.append("exclude", prop.excludeTags.join(","));
  }
  const url = params.toString() ? `/graph?${params.toString()}` : `/graph`;

  fetch(url)
    .then((resp) => resp.json())
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
            size: Math.max(5, Math.min(20, node.num_links / 2)), // Cap max size
            color: nodeColor,
            borderColor: nodeBorderColor,
            borderSize: 2, // Consistent border size
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
    })
    .catch((error) => {
      console.error("Failed to load graph data:", error);
      const errorMsg =
        error.name === "TypeError" && error.message.includes("fetch")
          ? "Server is not responding. Please check if the server is running."
          : `Failed to load graph: ${error.message}`;
      emit("error", errorMsg);
    });
};

let layout: FA2Layout;

function setupGraph() {
  const settings = forceAtlas2.inferSettings(graph);
  layout = new FA2Layout(graph, {
    settings: settings,
  });

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
  try {
    const communities = louvain(graph);
    Object.entries(communities).forEach(([node, communityId]) => {
      const color = colors[communityId % colors.length];
      graph.mergeNodeAttributes(node, {
        community: communityId,
        color,
      });
    });
  } catch (e) {
    console.log("Community detection skipped:", e);
  }

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
    allowInvalidContainer: true,
    renderEdgeLabels: false,
    enableEdgeEvents: false,
    labelFont: "Arial, sans-serif",
    labelSize: 12,
    labelWeight: "500",
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
  removedNodes?: string[];
  removedLinks?: RoamLink[];
}) => {
  const style = window.getComputedStyle(document.body);
  const nodeColor = style.getPropertyValue("--node").trim();
  const nodeBorderColor = style.getPropertyValue("--node-border").trim();

  // Handle node removals first
  if (updates.removedNodes) {
    for (const nodeId of updates.removedNodes) {
      try {
        if (graph.hasNode(nodeId)) {
          graph.dropNode(nodeId);
          console.log(`Removed node: ${nodeId}`);
        }
      } catch (err) {
        console.log(`Error removing node ${nodeId}: ${err}`);
      }
    }
  }

  // Handle link removals
  if (updates.removedLinks) {
    for (const link of updates.removedLinks) {
      try {
        if (graph.hasEdge(link.from, link.to)) {
          graph.dropEdge(link.from, link.to);
          console.log(`Removed link: ${link.from} -> ${link.to}`);
        }
      } catch (err) {
        console.log(`Error removing link ${link.from} -> ${link.to}: ${err}`);
      }
    }
  }

  // Handle node additions and updates
  for (const node of updates.nodes) {
    try {
      if (graph.hasNode(node.id)) {
        // Update existing node only if values actually changed
        const currentAttrs = graph.getNodeAttributes(node.id);
        const newSize = Math.max(5, Math.min(20, node.num_links / 2));

        if (
          currentAttrs.label !== node.title ||
          currentAttrs.size !== newSize ||
          currentAttrs.color !== nodeColor ||
          currentAttrs.borderColor !== nodeBorderColor
        ) {
          graph.mergeNodeAttributes(node.id, {
            label: node.title,
            size: newSize,
            color: nodeColor,
            borderColor: nodeBorderColor,
            borderSize: 2,
          });
        }
        console.log(`Updated node: ${node.id} (${node.title})`);
      } else {
        // Add new node with better positioning
        const x = randomNumber(-50, 50);
        const y = randomNumber(-50, 50);
        graph.addNode(node.id, {
          label: node.title,
          x: x,
          y: y,
          size: Math.max(5, Math.min(20, node.num_links / 2)),
          color: nodeColor,
          borderColor: nodeBorderColor,
          borderSize: 2,
          type: "bordered", // Ensure it uses the same type as other nodes
        });
        console.log(
          `Added new node: ${node.id} (${node.title}) at position (${x}, ${y})`,
        );
        console.log(`Node attributes:`, graph.getNodeAttributes(node.id));
        console.log(`Graph now has ${graph.order} nodes total`);
      }
    } catch (err) {
      console.log(`Error with node ${node.id} (${node.title}): ${err}`);
    }
  }

  // Handle link additions - check that both nodes exist
  const edgeColor = style.getPropertyValue("--overlay");
  for (const link of updates.links) {
    try {
      if (!graph.hasNode(link.from)) {
        console.log(
          `Cannot add link: source node '${link.from}' does not exist in graph`,
        );
        continue;
      }
      if (!graph.hasNode(link.to)) {
        console.log(
          `Cannot add link: target node '${link.to}' does not exist in graph`,
        );
        continue;
      }
      if (graph.hasEdge(link.from, link.to)) {
        console.log(`Link ${link.from} -> ${link.to} already exists, skipping`);
        continue;
      }

      graph.addEdge(link.from, link.to, { color: edgeColor });
      console.log(`✅ Successfully added link: ${link.from} -> ${link.to}`);
    } catch (error) {
      console.log(`❌ Error adding link ${link.from} -> ${link.to}: ${error}`);
    }
  }

  console.log(`Link processing complete. Graph now has ${graph.size} edges`);

  // Log all nodes in the graph for debugging
  console.log("All nodes in graph:", graph.nodes());

  // Only rebuild the graph if we have significant changes
  const hasSignificantChanges =
    (updates.removedNodes && updates.removedNodes.length > 0) ||
    (updates.removedLinks && updates.removedLinks.length > 0) ||
    updates.nodes.length > 10 || // Threshold for rebuilding
    updates.links.length > 10;

  if (hasSignificantChanges) {
    document.getElementById("graph")!.innerHTML = "";
    setupGraph();
  } else {
    // Force a complete refresh for new nodes to ensure they're properly rendered and clickable
    if (
      updates.nodes.length > 0 ||
      (updates.removedNodes && updates.removedNodes.length > 0)
    ) {
      console.log(
        "New or removed nodes detected - performing complete sigma refresh",
      );
      console.log(`Graph now has ${graph.order} nodes and ${graph.size} edges`);

      if (graph.order > 0) {
        console.log("Running community detection for updated graph");
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

        try {
          const communities = louvain(graph);
          Object.entries(communities).forEach(([node, communityId]) => {
            const color = colors[communityId % colors.length];
            graph.mergeNodeAttributes(node, {
              community: communityId,
              color,
            });
          });
          console.log("Community detection completed");
        } catch (e) {
          console.log("Community detection skipped:", e);
        }
      }

      if (sigma) {
        // Destroy and recreate sigma to ensure new nodes are properly integrated
        sigma.kill();

        const element = document.getElementById("graph")!;
        const style = window.getComputedStyle(document.body);
        const textColor = style.getPropertyValue("--text");

        sigma = new Sigma(graph, element, {
          defaultNodeType: "bordered",
          nodeProgramClasses: {
            bordered: NodeBorderProgram,
          },
          defaultDrawNodeHover: drawHover,
          labelColor: { color: textColor },
          allowInvalidContainer: true,
          renderEdgeLabels: false,
          enableEdgeEvents: false,
          labelFont: "Arial, sans-serif",
          labelSize: 12,
          labelWeight: "500",
        });

        // Re-bind click events for all nodes (including new ones)
        sigma.on("downNode", (e) => {
          zoomOnto(e.node, old_node);
          emit("openNode", e.node);
        });

        console.log("Sigma instance recreated with click handlers");

        // Always restart layout for new nodes to help them find their position
        console.log("Restarting layout to position new nodes");
        layout.start();
        setTimeout(() => {
          if (generalSettings.stopLayoutAfter != null) {
            console.log("Auto-stopping layout after positioning new nodes");
            layout.stop();
          }
        }, 5000); // Give more time for nodes to settle into good positions
      }
    } else {
      // Just refresh for minor changes
      if (sigma) {
        console.log("Minor changes - just refreshing sigma");
        sigma.refresh();
      }
    }
  }

  // Maintain zoom on current node if it still exists
  if (old_node && graph.hasNode(old_node)) {
    zoomOnto(old_node, old_node);
  }
};

const prop = defineProps<{
  count: number;
  toggleLayouter: boolean;
  zoomNode: string;
  updates: {
    nodes: RoamNode[];
    links: RoamLink[];
    removedNodes?: string[];
    removedLinks?: RoamLink[];
  } | null;
  filterTags?: string[];
  excludeTags?: string[];
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
    console.log("STARTING TO UPDATE GRAPH", {
      nodes: prop.updates.nodes.length,
      links: prop.updates.links.length,
      removedNodes: prop.updates.removedNodes?.length || 0,
      removedLinks: prop.updates.removedLinks?.length || 0,
    });
    incrementalGraphUpdate(prop.updates);

    // Emit event to clear the updates ref and ensure reactivity
    emit("updatesProcessed");
  }
});

onMounted(updateGraph);
const emit = defineEmits(["openNode", "updatesProcessed", "error"]);
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
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 97%, var(--surface))
  );
  border-radius: 0;
  transition: background 0.3s ease;
}

/* Add subtle grid pattern for depth */
#graph::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: radial-gradient(
    circle at 1px 1px,
    color-mix(in srgb, var(--overlay) 15%, transparent) 1px,
    transparent 0
  );
  background-size: 30px 30px;
  pointer-events: none;
  opacity: 0.3;
  z-index: 1;
}

/* Ensure graph canvas is above the grid */
#graph canvas {
  position: relative;
  z-index: 2;
}
</style>
