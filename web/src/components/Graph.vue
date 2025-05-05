<script setup lang="ts">
import { MultiGraph } from "graphology";
import louvain from "graphology-communities-louvain";
import forceAtlas2 from "graphology-layout-forceatlas2";
import FA2Layout from "graphology-layout-forceatlas2/worker";
import Sigma from "sigma";
import { NodeBorderProgram } from "@sigma/node-border";
import type GraphData from "@/types";
import { onMounted } from "vue";

const randomNumber = (min: number, max: number) =>
  Math.random() * (max - min) + min;
const graph = new MultiGraph();

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
        (node: { title: string; id: string; parent: string }) => {
          graph.addNode(node.id, {
            label: node.title,
            x: randomNumber(1, 100),
            y: randomNumber(1, 100),
            size: 5,
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

function setupGraph() {
  const settings = forceAtlas2.inferSettings(graph);
  const layout = new FA2Layout(graph, {
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
  const sigma = new Sigma(graph, element, {
    defaultNodeType: "bordered",
    nodeProgramClasses: {
      bordered: NodeBorderProgram,
    },
    labelColor: { color: textColor },
  });

  sigma.on("downNode", (e) => {
    const node = e.node;
    console.log(node);
    // preview(node);
  });
}

onMounted(updateGraph);
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
