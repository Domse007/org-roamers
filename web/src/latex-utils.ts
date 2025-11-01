/**
 * Secure LaTeX processing utilities for org-roamers
 *
 * This new implementation requests LaTeX rendering by node ID and block index
 * instead of sending raw LaTeX to the server, providing much better security.
 */

// Debug mode for LaTeX processing
let debugMode = false;

export function setLatexDebugMode(enabled: boolean): void {
  debugMode = enabled;
  console.log(`LaTeX debug mode ${enabled ? "enabled" : "disabled"}`);
}

function debugLog(...args: any[]): void {
  if (debugMode) {
    console.log("[LaTeX Debug]", ...args);
  }
}

/**
 * Process LaTeX placeholders in the content and replace them with rendered SVG
 */
export async function processLatexPlaceholders(
  element: HTMLElement,
  nodeId: string,
  latexBlocks: string[],
): Promise<void> {
  debugLog("Processing LaTeX placeholders", {
    nodeId,
    blockCount: latexBlocks.length,
  });

  // Find all LaTeX placeholders in the element
  const placeholders = element.querySelectorAll(
    ".org-latex-placeholder, .org-latex-block-placeholder",
  );

  debugLog("Found placeholders:", placeholders.length);

  for (let i = 0; i < placeholders.length; i++) {
    const placeholder = placeholders[i];
    const latexIndex = parseInt(
      placeholder.getAttribute("data-latex-index") || "0",
    );

    debugLog(`Processing placeholder ${i}, latex index ${latexIndex}`);

    if (latexIndex < 0 || latexIndex >= latexBlocks.length) {
      console.warn(`Invalid LaTeX index ${latexIndex}, skipping`);
      continue;
    }

    try {
      await renderLatexBlock(nodeId, latexIndex, placeholder);
    } catch (error) {
      console.error(`Failed to render LaTeX block ${latexIndex}:`, error);

      // Show error in the placeholder
      placeholder.innerHTML = `
        <span class="latex-error">
          [LaTeX Error: Block ${latexIndex}]
        </span>
      `;
    }
  }
}

/**
 * Render a specific LaTeX block by requesting it from the server
 */
async function renderLatexBlock(
  nodeId: string,
  latexIndex: number,
  placeholder: Element,
): Promise<void> {
  debugLog(`Rendering LaTeX block ${latexIndex} for node ${nodeId}`);

  // Get current theme color
  const style = window.getComputedStyle(document.body);
  const textColor = style.getPropertyValue("--text") || "#c6d0f5";
  const colorEncoded = encodeURIComponent(textColor.replace("#", ""));

  // Request LaTeX rendering by index
  const url = `/latex?id=${encodeURIComponent(nodeId)}&index=${latexIndex}&color=${colorEncoded}&scope=file`;
  debugLog("Requesting:", url);

  const response = await fetch(url, {
    credentials: "include", // Include cookies for authentication
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Server error ${response.status}: ${errorText}`);
  }

  const svg = await response.text();
  debugLog(
    `Received SVG for block ${latexIndex}:`,
    svg.substring(0, 200) + "...",
  );

  if (
    !svg ||
    (!svg.trim().startsWith("<svg") && !svg.trim().includes("<svg"))
  ) {
    throw new Error("Invalid SVG response from server");
  }

  // Create a temporary container to parse the SVG
  const tempContainer = document.createElement("div");
  tempContainer.innerHTML = svg;

  // Find the SVG element (it might not be the first child if there's an XML declaration)
  const svgElement = tempContainer.querySelector("svg");

  if (!svgElement) {
    throw new Error("Failed to parse SVG response");
  }

  // Add classes and styling to the SVG for theme integration
  svgElement.classList.add("org-latex-rendered");
  svgElement.setAttribute(
    "style",
    "fill: var(--text); max-width: 100%; height: auto;",
  );

  // Replace the placeholder directly in the DOM
  placeholder.parentNode?.replaceChild(svgElement, placeholder);

  debugLog(`Successfully replaced LaTeX block ${latexIndex} in DOM`);
}

/**
 * Test function for debugging LaTeX rendering
 */
export function testLatexRendering(nodeId: string, latexIndex: number): void {
  console.log("=== TESTING LATEX RENDERING ===");
  console.log("Node ID:", nodeId);
  console.log("LaTeX Index:", latexIndex);

  const mockPlaceholder = document.createElement("div");
  mockPlaceholder.className = "org-latex-placeholder";
  mockPlaceholder.setAttribute("data-latex-index", latexIndex.toString());
  mockPlaceholder.innerHTML = `[LaTeX Block ${latexIndex}]`;

  // Add to document body for testing
  document.body.appendChild(mockPlaceholder);

  renderLatexBlock(nodeId, latexIndex, mockPlaceholder)
    .then(() => console.log("✅ Test completed successfully"))
    .catch((error) => console.error("❌ Test failed:", error));

  console.log("=== END TEST ===");
}
