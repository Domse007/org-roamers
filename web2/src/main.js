import './style.css'

import renderMathInElement from 'katex/dist/contrib/auto-render';

import hljs from 'highlight.js';
import { MultiGraph } from "graphology";
// import forceAtlas2 from "graphology-layout-forceatlas2";
// TODO:
import FA2Layout from "graphology-layout-forceatlas2/worker";
import Sigma from "sigma";

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

export const preview = (name) => {
  fetch(`/org?title=${name}`)
    .then((response) => {
      return response.text();
    }).then((html) => {
      document.getElementById('org-preview').innerHTML = html;
      renderMathInElement(document.getElementById('org-preview'));
      syntaxHighlightSite();
    });	  
};

const randomNumber = (min, max) => Math.random() * (max - min) + min;

// Create a graphology graph
const graph = new MultiGraph();

const updateGraph = () => {
  fetch(`/graph`)
    .then((resp) => resp.json())
    .then((text) => JSON.parse(text))
    .then((json) => {
      json["nodes"].forEach((node) => {
	graph.addNode(node, {
          label: node,
          x: randomNumber(1, 100),
          y: randomNumber(1, 100),
          size: 10,
          color: "blue"
        });
      });
      json["edges"].forEach((edge) => {
        try {
          graph.addEdge(edge[0], edge[1]);
        } catch (error) {
          console.log(`${edge[0]}->${edge[1]}: ${error}`);
        }
      });
      setupGraph()
    })
}

export function setupGraph() {
  const layout = new FA2Layout(graph, {
    settings: {gravity: 0}
  });
  
  layout.start();
  const sigmaInstance = new Sigma(graph, document.getElementById("graph"));
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
      searchSuggestion.innerHTML += `<div class="suggestion" style="padding: 5px; cursor: pointer;" onmouseout="this.style.background='gray';" onmouseover="this.style.background='lightgray';">${e.title}</div>`;
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
