import hljs from "highlight.js";

/**
 * Initialize and configure syntax highlighting
 * Should be called once during application initialization
 */
export function setupSyntaxHighlighting() {
  // Dynamic import for COBOL syntax highlighting to avoid build issues
  import("highlightjs-cobol")
    .then((hljsCOBOL) => {
      hljs.registerLanguage("cobol", hljsCOBOL.default);
    })
    .catch((error) => {
      console.warn("Failed to load COBOL syntax highlighting:", error);
    });

  // Dynamic import for Lisp syntax highlighting and register elisp/emacs-lisp aliases
  import("highlight.js/lib/languages/lisp")
    .then((lispLang) => {
      hljs.registerLanguage("lisp", lispLang.default);
      hljs.registerLanguage("elisp", lispLang.default);
      hljs.registerLanguage("emacs-lisp", lispLang.default);
      console.log("Registered lisp, elisp, and emacs-lisp highlighting");
    })
    .catch((error) => {
      console.warn("Failed to load Lisp syntax highlighting:", error);
      // Fallback: try to register aliases if lisp is already available
      try {
        if (hljs.getLanguage("lisp")) {
          hljs.registerAliases("elisp", { languageName: "lisp" });
          hljs.registerAliases("emacs-lisp", { languageName: "lisp" });
          console.log(
            "Registered elisp and emacs-lisp aliases for existing lisp",
          );
        }
      } catch (aliasError) {
        console.warn(
          "Failed to register lisp aliases as fallback:",
          aliasError,
        );
      }
    });

  // Register JCL syntax highlighting
  // https://amyfare.ca/files/jcl.min.js
  hljs.registerLanguage(
    "jcl",
    (function () {
      "use strict";

      return function (e: any) {
        return {
          aliases: ["jcl"],
          case_insensitive: false,

          contains: [
            e.COMMENT(/^\/\/\*/, /$/),
            {
              begin: /^(?=\/\/)/,
              end: /$/,

              keywords: {
                $pattern: /(?<= )\w+(?= )/,
                keyword:
                  "COMMAND CNTL DD ENDCNTL EXEC IF THEN ELSE ENDIF INCLUDE JCLLIB " +
                  "JOB OUTPUT PEND PROC SET XMIT",
              },

              contains: [
                {
                  scope: "symbol",
                  match: /\/\/(\w+\.?)*/,
                },
                {
                  scope: "parameter",
                  match: /(?<=[ ,])\w+(?=\=)/,
                },
                {
                  scope: "operator",
                  match: "[=|<>]",
                },
                {
                  scope: "punctuation",
                  match: "[()]",
                },
                {
                  scope: "variable",
                  match: /(?<!&)&\w+/,
                },

                e.APOS_STRING_MODE,
                e.NUMBER_MODE,
              ],
            },
            {
              scope: "meta",
              begin: /^\/\*/,
              end: /$/,
            },
            {
              scope: "literal",
            },
          ],
        };
      };
    })(),
  );

  // Update the selector from 'pre code' to 'code' to autodetect inline src
  // like src_java[:exports code]{ void main() } which has no <pre></pre>.
  hljs.configure({ cssSelector: "code" });
}
