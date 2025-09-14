import renderMathInElement from "katex/contrib/auto-render";

/**
 * Simple LaTeX processing utilities for org-roamers
 */

// Debug mode for LaTeX processing
let debugMode = false;

export function setLatexDebugMode(enabled: boolean): void {
  debugMode = enabled;
  console.log(`LaTeX debug mode ${enabled ? 'enabled' : 'disabled'}`);
}

function debugLog(...args: any[]): void {
  if (debugMode) {
    console.log('[LaTeX Debug]', ...args);
  }
}

/**
 * Test function to manually trigger server fallback (for debugging)
 */
export function testServerFallback(
  latex: string,
  currentId: string, 
  updateContent: (newContent: string) => void,
  getCurrentContent: () => string
): void {
  console.log("=== MANUAL SERVER FALLBACK TEST ===");
  console.log("Testing with LaTeX:", latex);
  
  const callback = createServerFallbackErrorCallback(currentId, updateContent, getCurrentContent);
  const fakeError = new Error(`Manual test of LaTeX: ${latex}`);
  callback(`Manual test error for: ${latex}`, fakeError);
  
  console.log("=== END TEST ===");
}



/**
 * Basic macros for common math notation
 */
export const DEFAULT_MACROS = {
  "\\RR": "\\mathbb{R}",
  "\\NN": "\\mathbb{N}",
  "\\ZZ": "\\mathbb{Z}",
  "\\QQ": "\\mathbb{Q}",
  "\\CC": "\\mathbb{C}",
};

/**
 * Simple KaTeX configuration
 */
export const KATEX_CONFIG = {
  throwOnError: false,
  trust: true,
  strict: false,
  output: 'html' as const,
  macros: DEFAULT_MACROS,
};

/**
 * LaTeX delimiters for KaTeX - includes all common environments
 * Unsupported ones will fallback to server rendering
 */
export const LATEX_DELIMITERS = [
  { left: "$$", right: "$$", display: true },
  { left: "\\(", right: "\\)", display: false },
  { left: "\\[", right: "\\]", display: true },
  { left: "\\begin{equation}", right: "\\end{equation}", display: true },
  { left: "\\begin{align}", right: "\\end{align}", display: true },
  { left: "\\begin{align*}", right: "\\end{align*}", display: true },
  { left: "\\begin{matrix}", right: "\\end{matrix}", display: true },
  { left: "\\begin{pmatrix}", right: "\\end{pmatrix}", display: true },
  { left: "\\begin{cases}", right: "\\end{cases}", display: true },
  // Unsupported environments (will go to server)
  { left: "\\begin{algorithm}", right: "\\end{algorithm}", display: true },
  { left: "\\begin{algorithmic}", right: "\\end{algorithmic}", display: true },
  { left: "\\begin{tikzpicture}", right: "\\end{tikzpicture}", display: true },
  { left: "\\begin{lstlisting}", right: "\\end{lstlisting}", display: true },
];

/**
 * Creates KaTeX auto-render config with error callback for server fallback
 */
export function createAutoRenderConfig(errorCallback?: (message: string, err: Error) => void) {
  const config = {
    ...KATEX_CONFIG,
    delimiters: LATEX_DELIMITERS,
    ...(errorCallback && { errorCallback }),
  };
  
  console.log("KaTeX auto-render config:", config);
  console.log("Delimiters:", LATEX_DELIMITERS);
  
  return config;
}

/**
 * Server fallback for unsupported LaTeX - sends to /latex endpoint
 */
export function createServerFallbackErrorCallback(
  currentId: string,
  updateContent: (newContent: string) => void,
  getCurrentContent: () => string
) {
  return (message: string, err: Error) => {
    console.log("🔥 Server fallback triggered:", message);
    
    // Try to extract the specific failed LaTeX from the error message
    const content = getCurrentContent();
    let latex = "";
    
    // Method 1: Extract from our manual trigger message
    if (message.includes("Unprocessed algorithmic block:")) {
      const match = message.match(/Unprocessed algorithmic block: (.+)$/s);
      if (match) {
        latex = match[1];
        console.log("📝 Extracted from manual message:", latex.substring(0, 100) + "...");
      }
    }
    
    // Method 2: Fall back to pattern matching if extraction failed
    if (!latex) {
      console.log("📝 Using pattern matching fallback");
      const patterns = [
        /\\begin\{algorithmic\}[\s\S]*?\\end\{algorithmic\}/g,
        /\\begin\{[^}]+\}[\s\S]*?\\end\{[^}]+\}/g,
        /\$\$[^$]+\$\$/g,
        /\\\[[^\]]+\\\]/g
      ];
      
      for (const pattern of patterns) {
        const matches = content.match(pattern);
        if (matches) {
          latex = matches[0];
          console.log("📝 Found with pattern:", pattern.toString());
          break;
        }
      }
    }
    
    if (!latex) {
      console.warn("❌ No LaTeX found for server fallback");
      return;
    }
    
    console.log("📤 Sending LaTeX to server:", latex.substring(0, 100) + "...");
    
    // Send to server with current theme color
    const encoded = encodeURIComponent(latex);
    const style = window.getComputedStyle(document.body);
    const textColor = style.getPropertyValue("--text") || "#c6d0f5";
    const colorEncoded = encodeURIComponent(textColor.replace('#', ''));
    
    const url = `/latex?tex=${encoded}&color=${colorEncoded}&id=${encodeURIComponent(currentId)}`;
    console.log("🌐 Server URL:", url);
    console.log("🎨 Color:", textColor);
    
    fetch(url)
      .then(resp => {
        console.log("📥 Server response status:", resp.status, resp.statusText);
        if (!resp.ok) {
          return resp.text().then(errorText => {
            throw new Error(`Server error ${resp.status}: ${errorText}`);
          });
        }
        return resp.text();
      })
      .then(svg => {
        console.log("📄 Server returned:", typeof svg, svg.length, "chars");
        console.log("📄 First 200 chars:", svg.substring(0, 200));
        
        if (svg && (svg.trim().startsWith('<svg') || svg.includes('<svg'))) {
          console.log("✅ Valid SVG detected");
          const currentContent = getCurrentContent();
          console.log("🔄 Current content length:", currentContent.length);
          
          const newContent = currentContent.replace(latex, svg);
          console.log("🔄 New content length:", newContent.length);
          console.log("🔄 Content changed:", newContent !== currentContent);
          
          if (newContent !== currentContent) {
            updateContent(newContent);
            console.log("✅ ✅ Successfully replaced with server SVG!");
          } else {
            console.warn("⚠️ Content didn't change after replacement");
            console.log("🔍 Looking for LaTeX in content:", currentContent.includes(latex.substring(0, 50)));
          }
        } else {
          console.warn("❌ Server didn't return valid SVG");
          console.log("❌ Response starts with:", svg.substring(0, 50));
        }
      })
      .catch(error => {
        console.error("💥 Server fallback failed:", error);
        console.error("💥 Error details:", error.message);
      });
  };
}

