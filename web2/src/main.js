import './style.css'

import renderMathInElement from 'katex/dist/contrib/auto-render';

import hljs from 'highlight.js';
import { MultiGraph } from "graphology";
// import forceAtlas2 from "graphology-layout-forceatlas2";
// TODO:
import forceAtlas2 from 'graphology-layout-forceatlas2';
import FA2Layout from "graphology-layout-forceatlas2/worker";
import Sigma from "sigma";
import { NodeBorderProgram } from "@sigma/node-border";

const MIN_SIZE = 200;
const BORDER_SIZE = 10;
const panel = document.getElementById('org-preview-dragging-area');
const panel_wrapper = document.getElementById('org-preview-wrapper');

let m_pos;

export function resize(e) {
  const dx = m_pos - e.x;
  m_pos = e.x;
  const new_width = Math.max((parseInt(getComputedStyle(panel_wrapper, '').width) + dx), MIN_SIZE);
  panel_wrapper.style.width = new_width + "px";
}

export function setupPreview() {
  panel.addEventListener("mousedown", function(e){
    if (e.offsetX < BORDER_SIZE) {
      m_pos = e.x;
      document.addEventListener("mousemove", resize, false);
    }
  }, false);

  document.addEventListener("mouseup", function(){
    document.removeEventListener("mousemove", resize, false);
  }, false);
}

export const syntaxHighlightSite = () => {
  Array.from(document.getElementsByClassName("src"))
    .forEach((item) => {
      hljs.highlightElement(item)
    });
  hljs.highlightAll();
}

const katexOptions = {
  delimiters: [
    {left: "$$", right: "$$", display: true},
    {left: "\\(", right: "\\)", display: false},
    {left: "\\begin{equation}", right: "\\end{equation}", display: true},
    {left: "\\begin{align}", right: "\\end{align}", display: true},
    {left: "\\begin{align*}", right: "\\end{align*}", display: true},
    {left: "\\begin{alignat}", right: "\\end{alignat}", display: true},
    {left: "\\begin{gather}", right: "\\end{gather}", display: true},
    {left: "\\begin{CD}", right: "\\end{CD}", display: true},
    {left: "\\begin{algorithm}", right: "\\end{algorithm}", display: true},
    {left: "\\begin{algorithmic}", right: "\\end{algorithmic}", display: true},
    {left: "\\[", right: "\\]", display: true}
  ],
  errorCallback: (message, stack) => {
    const latex = message.substring(36, message.length - 7);
    const encoded = encodeURI(latex);
    const style = window.getComputedStyle(document.body);
    const textColor = style.getPropertyValue('--text');
    const colorEncoded = encodeURI(textColor.substring(1));
    fetch(`/latex?tex=${encoded}&color=${colorEncoded}`)
      .then((resp) => resp.text())
      .then((svg) => {
        const container = document.getElementById('org-preview');
        let newHTML = container.innerHTML.replace(latex, svg);
        container.innerHTML = newHTML;
      });
  }
};

export const preview = (name) => {
  fetch(`/org?title=${name}`)
    .then((response) => {
      return response.text();
    }).then((html) => {
      document.getElementById('org-preview').innerHTML = html;
      renderMathInElement(document.getElementById('org-preview'), katexOptions);
      syntaxHighlightSite();
    });	  
};

const randomNumber = (min, max) => Math.random() * (max - min) + min;

// Create a graphology graph
const graph = new MultiGraph();

const updateGraph = () => {
  const style = window.getComputedStyle(document.body);
  const nodeColor = style.getPropertyValue('--node');
  const edgeColor = style.getPropertyValue('--overlay');
  const nodeBorderColor = style.getPropertyValue('--node-border');
  
  fetch(`/graph`)
    .then((resp) => resp.json())
    .then((text) => JSON.parse(text))
    .then((json) => {
      json["nodes"].forEach((node) => {
	graph.addNode(node[0], {
          label: node[1],
          x: randomNumber(1, 100),
          y: randomNumber(1, 100),
          size: 10,
          color: nodeColor,
          borderColor: nodeBorderColor,
        });
      });
      json["edges"].forEach((edge) => {
        try {
          graph.addEdge(edge[0], edge[1], { color: edgeColor });
        } catch (error) {
          console.log(`${edge[0]}->${edge[1]}: ${error}`);
        }
      });
      setupGraph()
    })
}

export function setupGraph() {
  const settings = forceAtlas2.inferSettings(graph);
  const layout = new FA2Layout(graph, {
    settings: settings
  });
  
  layout.start();
  let sigma = new Sigma(graph, document.getElementById("graph"), {
    defaultNodeType: "bordered",
    nodeProgramClasses: {
      bordered: NodeBorderProgram,
    },
  });

  const style = window.getComputedStyle(document.body);
  const textColor = style.getPropertyValue('--text');
  sigma.settings.labelColor = { color: textColor };

  sigma.on("downNode", (e) => {
    const node = e.node;
    console.log(node);
    preview(node);
  });
}

const search = async (query) => {
  const resp = await fetch(`/search?q=${query}`);
  const text = await resp.json();
  const res = JSON.parse(text);
  return res;
}

let searchInput = document.getElementById('search-input');
let searchSuggestion = document.getElementById('search-suggestion-wrapper');

const InputHandler = (event) => {
  const query = searchInput.value;
  searchSuggestion.innerHTML = "";
  search(query).then((res) => {
    res["results"].forEach((e) => {
      searchSuggestion.innerHTML += `<div class="suggestion" style="padding: 5px; cursor: pointer;">${e.title}</div>`;
    })
    updateSuggestions();
  })
};

const updateSuggestions = () => {
  Array.from(document.getElementsByClassName("suggestion")).forEach((e) => {
    e.onclick = () => {
      preview(e.textContent);
    };
  });
};

export function setupSearchBarEventListener() {
  searchInput = document.getElementById('search-input');
  searchSuggestion = document.getElementById('search-suggestion-wrapper');
  
  searchInput.addEventListener("input", InputHandler);
  searchInput.addEventListener("focus", InputHandler);

  searchInput.addEventListener("blur", () => {
    setTimeout(() => {
      searchSuggestion.innerHTML = "";
    }, 250);
  });
}

document.addEventListener('DOMContentLoaded', function() {
  setupPreview()
  preview("Kreise");
  setupSearchBarEventListener();
  updateGraph();
});

const hiddenDiv = document.getElementById("search-wrapper");
let isMouseOverDiv = false;

// Show the div when mouse is at the top
window.addEventListener("mousemove", function(event) {
  if (event.clientY < 50 || isMouseOverDiv) {
    hiddenDiv.style.top = "0"; // Slide in
  } else {
    hiddenDiv.style.top = "-100px"; // Slide out
  }
});

// Prevent hiding when mouse is over the div
hiddenDiv.addEventListener("mouseenter", function() {
  isMouseOverDiv = true;
});

// Allow hiding when mouse leaves the div
hiddenDiv.addEventListener("mouseleave", function() {
  isMouseOverDiv = false;
});
