import { ref, type Ref } from "vue";
import { getScope } from "../settings";
import { type OrgAsHTMLResponse } from "../types";
import { History } from "../history";
import { getRouter } from "../router";

/**
 * Composable for handling preview content fetching and management
 */
export function usePreviewContent() {
  const shown: Ref<"none" | "flex"> = ref("none");
  const links: Ref<{ display: string; id: string }[]> = ref([]);
  const incomingLinks: Ref<{ display: string; id: string }[]> = ref([]);
  const tags: Ref<string[]> = ref([]);
  const rendered = ref("");

  let current_id: string = "";
  let current_latex_blocks: string[] = [];

  const history = new History<string>();
  const router = getRouter();

  const preview = (
    id: string,
    onError?: (message: string) => void,
    updateRouter: boolean = true,
  ): Promise<void> => {
    current_id = id;
    console.log(`Previewing ${id}`);
    const scope: "file" | "node" = getScope();

    return fetch(`/org?id=${id}&scope=${scope}`)
      .then((response) => {
        console.log("Org response status:", response.status);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        return response.json();
      })
      .then((resp: OrgAsHTMLResponse) => {
        console.log("Org response data:", {
          orgLength: resp.org?.length || 0,
          linksCount: resp.outgoing_links?.length || 0,
          latexBlocksCount: resp.latex_blocks?.length || 0,
        });
        history.push(id);
        rendered.value = resp.org;
        links.value = resp.outgoing_links;
        incomingLinks.value = resp.incoming_links || [];
        tags.value = resp.tags || [];
        current_latex_blocks = resp.latex_blocks || [];
        console.log(
          `Loaded content with ${current_latex_blocks.length} LaTeX blocks`,
        );

        // Update browser history via router
        if (updateRouter) {
          router.push(id);
        }

        expand();
      })
      .catch((error) => {
        console.error("Failed to load org content:", error);
        const errorMsg =
          error.name === "TypeError" && error.message.includes("fetch")
            ? "Server is not responding. Please check if the server is running."
            : `Failed to load content: ${error.message}`;

        if (onError) {
          onError(errorMsg);
        }

        rendered.value = `<div class="error">${errorMsg}</div>`;
        expand();
      });
  };

  const expand = () => {
    shown.value = "flex";
  };

  const collapse = () => {
    shown.value = "none";
  };

  const toggle = () => {
    if (rendered.value.length === 0) {
      return;
    }
    if (shown.value === "none") {
      expand();
    } else {
      collapse();
    }
  };

  const getCurrentId = () => current_id;
  const getLatexBlocks = () => current_latex_blocks;

  return {
    // State
    shown,
    links,
    incomingLinks,
    tags,
    rendered,
    history,

    // Methods
    preview,
    expand,
    collapse,
    toggle,
    getCurrentId,
    getLatexBlocks,
  };
}
