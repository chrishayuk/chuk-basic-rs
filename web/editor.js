// Makes the page's plain text box understand BASIC, with no library: the
// text is coloured in a layer underneath the box, problems are underlined
// and listed, and resting the pointer on a word says what it does.
//
// Everything it knows comes from the language server (lsp-worker.js), the
// same one an editor talks to. Completion is left to real editors.

/** The order of the server's semantic token legend. */
export const TOKEN_TYPES = ["comment", "string", "number", "variable", "function", "keyword", "operator", "label"];

const escape = (text) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/**
 * The program as HTML, one string per line: a span for each coloured token,
 * and inside or around it a span for each stretch that has a problem.
 *
 * @param {string} text
 * @param {number[]} data  the server's five numbers per token
 * @param {{range: {start: {line: number, character: number}, end: {line: number, character: number}}, severity: number}[]} diagnostics
 * @returns {string[]}
 */
export function renderLines(text, data = [], diagnostics = []) {
  const lines = text.split("\n");
  // per line: [from, to, class] pieces, tokens first
  const marks = lines.map(() => []);
  let line = 0;
  let character = 0;
  for (let i = 0; i + 4 < data.length; i += 5) {
    line += data[i];
    character = data[i] ? data[i + 1] : character + data[i + 1];
    if (line < lines.length && TOKEN_TYPES[data[i + 3]]) {
      marks[line].push([character, character + data[i + 2], `tok-${TOKEN_TYPES[data[i + 3]]}`]);
    }
  }
  for (const problem of diagnostics) {
    const { start, end } = problem.range;
    if (start.line >= lines.length) continue;
    const to = end.line === start.line ? Math.max(end.character, start.character + 1) : lines[start.line].length;
    marks[start.line].push([start.character, to, problem.severity === 1 ? "problem-error" : "problem-warning"]);
  }
  return lines.map((content, n) => {
    // cut the line wherever any mark starts or ends, and class each piece
    const cuts = new Set([0, content.length]);
    for (const [from, to] of marks[n]) {
      cuts.add(Math.min(from, content.length));
      cuts.add(Math.min(to, content.length));
    }
    const points = [...cuts].sort((a, b) => a - b);
    let html = "";
    for (let i = 0; i + 1 < points.length; i++) {
      const [from, to] = [points[i], points[i + 1]];
      const classes = marks[n].filter(([a, b]) => a <= from && to <= b).map(([, , name]) => name);
      const piece = escape(content.slice(from, to));
      html += classes.length ? `<span class="${classes.join(" ")}">${piece}</span>` : piece;
    }
    return html;
  });
}

/** The server's small Markdown as HTML: code blocks, paragraphs, emphasis. */
export function renderHover(markdown) {
  return markdown
    .replace(/```\n(?!\n)/g, "```\n\n")
    .split(/\n{2,}/)
    .map((block) => {
      const fence = /^```\w*\n([\s\S]*?)\n?```$/.exec(block.trim());
      if (fence) return `<pre>${escape(fence[1])}</pre>`;
      return `<p>${escape(block).replace(/\*([^*]+)\*/g, "<em>$1</em>").replace(/`([^`]+)`/g, "<code>$1</code>")}</p>`;
    })
    .join("");
}

/**
 * @param {HTMLTextAreaElement} box   the program
 * @param {object} options
 * @param {HTMLElement} options.layer     a <pre> behind the box, for the colours
 * @param {HTMLElement} options.problems  where problems are listed
 * @param {HTMLElement} options.tip       the hover box
 * @param {URL} options.wasmUrl
 * @param {() => "later" | "dartmouth"} options.dialect
 * @returns {{ refresh: () => void }}
 */
export function attach(box, { layer, problems, tip, wasmUrl, dialect }) {
  let worker = null;
  let nextId = 0;
  const waiting = new Map();
  function ask(requests) {
    if (worker === null) {
      worker = new Worker(new URL("./lsp-worker.js", import.meta.url), { type: "module" });
      worker.onmessage = (event) => {
        const { id, error, ...answer } = event.data;
        waiting.get(id)?.(error ? null : answer);
        waiting.delete(id);
      };
      worker.onerror = () => {
        for (const resolve of waiting.values()) resolve(null);
        waiting.clear();
      };
    }
    const id = ++nextId;
    return new Promise((resolve) => {
      waiting.set(id, resolve);
      worker.postMessage({ id, wasmUrl: wasmUrl.href, text: box.value, dialect: dialect(), requests });
    });
  }

  function paint(html) {
    // a last empty line still needs height, as it has in the box
    layer.innerHTML = html.join("\n") + "\n";
    layer.scrollTop = box.scrollTop;
    layer.scrollLeft = box.scrollLeft;
  }

  let turn = 0;
  async function check() {
    const mine = ++turn;
    const text = box.value;
    const answer = await ask([{ method: "textDocument/semanticTokens/full" }]);
    if (mine !== turn || text !== box.value) return;
    if (answer === null) {
      // no server: plain text, and say nothing
      box.classList.remove("coloured");
      return;
    }
    paint(renderLines(text, answer.results[0]?.data || [], answer.diagnostics));
    box.classList.add("coloured");
    problems.replaceChildren(
      ...answer.diagnostics.slice(0, 4).map((problem) => {
        const item = document.createElement("button");
        item.type = "button";
        item.className = problem.severity === 1 ? "error" : "warning";
        item.textContent = `Line ${problem.range.start.line + 1}: ${problem.message}`;
        item.onclick = () => {
          const before = text.split("\n").slice(0, problem.range.start.line).join("\n").length;
          const at = before + (problem.range.start.line ? 1 : 0) + problem.range.start.character;
          box.focus();
          box.setSelectionRange(at, at + Math.max(1, problem.range.end.character - problem.range.start.character));
        };
        return item;
      }),
    );
    if (answer.diagnostics.length > 4) problems.append(`and ${answer.diagnostics.length - 4} more`);
  }

  let timer = 0;
  const refresh = () => {
    // show the new text at once, uncoloured, so the layers never disagree
    paint(box.value.split("\n").map(escape));
    clearTimeout(timer);
    timer = setTimeout(check, 100);
  };
  box.addEventListener("input", refresh);
  box.addEventListener("scroll", () => {
    layer.scrollTop = box.scrollTop;
    layer.scrollLeft = box.scrollLeft;
    tip.hidden = true;
  });

  // hover: which character is under the pointer, by measuring one
  const probe = document.createElement("span");
  probe.textContent = "0".repeat(50);
  probe.style.cssText = "position:absolute;visibility:hidden;white-space:pre";
  layer.after(probe);
  let rest = 0;
  box.addEventListener("mousemove", (event) => {
    clearTimeout(rest);
    tip.hidden = true;
    rest = setTimeout(async () => {
      const style = getComputedStyle(box);
      probe.style.font = style.font;
      const width = probe.getBoundingClientRect().width / 50;
      const height = parseFloat(style.lineHeight);
      const frame = box.getBoundingClientRect();
      const x = event.clientX - frame.left - parseFloat(style.paddingLeft) - parseFloat(style.borderLeftWidth) + box.scrollLeft;
      const y = event.clientY - frame.top - parseFloat(style.paddingTop) - parseFloat(style.borderTopWidth) + box.scrollTop;
      const position = { line: Math.floor(y / height), character: Math.floor(x / width) };
      const lines = box.value.split("\n");
      if (x < 0 || y < 0 || position.line >= lines.length || position.character >= lines[position.line].length) return;
      const answer = await ask([{ method: "textDocument/hover", params: { position } }]);
      const found = answer?.results[0];
      if (!found) return;
      tip.innerHTML = renderHover(found.contents.value);
      tip.hidden = false;
      const below = event.clientY + 18;
      tip.style.left = `${Math.max(8, Math.min(event.clientX, window.innerWidth - tip.offsetWidth - 8))}px`;
      tip.style.top = `${below + tip.offsetHeight > window.innerHeight ? event.clientY - tip.offsetHeight - 10 : below}px`;
    }, 350);
  });
  box.addEventListener("mouseleave", () => {
    clearTimeout(rest);
    tip.hidden = true;
  });

  refresh();
  return { refresh };
}
