const state = {
  search: "",
  stdlib: { modules: [] },
  explain: { fns: [], stages: [], seqs: [], types: [] },
};

async function loadDocs() {
  const [stdlibResp, explainResp] = await Promise.all([
    fetch("/api/stdlib"),
    fetch("/api/explain"),
  ]);
  state.stdlib = await stdlibResp.json();
  state.explain = await explainResp.json();
  render();
}

function normalize(text) {
  return String(text || "").toLowerCase();
}

function matchesSearch(name) {
  return normalize(name).includes(normalize(state.search));
}

function clearDetail() {
  document.getElementById("detail-view").innerHTML = "";
  document.getElementById("detail-empty").classList.remove("hidden");
}

function showDetail(title, signature, meta, bodyLines) {
  document.getElementById("detail-empty").classList.add("hidden");
  const detail = document.getElementById("detail-view");
  const metaHtml = meta
    .filter(Boolean)
    .map((line) => `<p class="meta-line">${line}</p>`)
    .join("");
  const bodyHtml = bodyLines
    .map((line) => `<li>${line}</li>`)
    .join("");
  detail.innerHTML = `
    <article class="card">
      <h2>${title}</h2>
      <pre class="signature">${signature || ""}</pre>
      ${metaHtml}
      <ul class="detail-list">${bodyHtml}</ul>
    </article>
  `;
}

function stdlibClickHandler(mod, fn) {
  const params = (fn.params || []).map((p) => `${p.name}: ${p.ty}`);
  const effects = (fn.effects || []).length ? `effects: ${(fn.effects || []).join(", ")}` : "effects: none";
  showDetail(`${mod.name}.${fn.name}`, fn.signature, [`returns: ${fn.returns}`, effects], params);
}

function projectClickHandler(kind, item) {
  const meta = [];
  if (item.effects && item.effects.length) {
    meta.push(`effects: ${item.effects.join(", ")}`);
  }
  if (item.file || item.line) {
    const location = [item.file, item.line].filter(Boolean).join(":");
    if (location) {
      meta.push(`source: ${location}`);
    }
  }
  const body = [];
  if (item.params) {
    item.params.forEach((p) => body.push(`${p.name}: ${p.ty}`));
  }
  if (item.bindings) {
    item.bindings.forEach((b) => body.push(`${b.slot} <- ${b.stage}`));
  }
  if (item.variants) {
    item.variants.forEach((v) => body.push(`variant ${v}`));
  }
  if (item.fields) {
    item.fields.forEach((f) => body.push(`${f.name}: ${f.ty}`));
  }
  showDetail(`${kind} ${item.name}`, item.signature || "", meta, body);
}

function renderStdlib(tree) {
  tree.innerHTML = "";
  state.stdlib.modules.forEach((mod) => {
    const functions = (mod.functions || []).filter((fn) => matchesSearch(`${mod.name}.${fn.name}`));
    if (!functions.length) {
      return;
    }

    const section = document.createElement("section");
    section.className = "group";
    section.innerHTML = `<h3>${mod.name}</h3>`;

    functions.forEach((fn) => {
      const button = document.createElement("button");
      button.className = "item";
      button.textContent = fn.name;
      button.addEventListener("click", () => stdlibClickHandler(mod, fn));
      section.appendChild(button);
    });

    tree.appendChild(section);
  });
}

function renderProject(tree) {
  tree.innerHTML = "";
  const groups = [
    ["Functions", state.explain.fns || []],
    ["Stages", state.explain.stages || []],
    ["Seqs", state.explain.seqs || []],
    ["Types", state.explain.types || []],
  ];

  let visibleCount = 0;
  groups.forEach(([label, items]) => {
    const filtered = items.filter((item) => matchesSearch(item.name));
    if (!filtered.length) {
      return;
    }

    visibleCount += filtered.length;
    const section = document.createElement("section");
    section.className = "group";
    section.innerHTML = `<h3>${label}</h3>`;

    filtered.forEach((item) => {
      const button = document.createElement("button");
      button.className = "item";
      button.textContent = item.name;
      button.addEventListener("click", () => projectClickHandler(label.slice(0, -1).toLowerCase(), item));
      section.appendChild(button);
    });

    tree.appendChild(section);
  });

  document.getElementById("project-section").classList.toggle("hidden", visibleCount === 0);
}

function render() {
  renderStdlib(document.getElementById("stdlib-tree"));
  renderProject(document.getElementById("project-tree"));
  clearDetail();
}

document.getElementById("search").addEventListener("input", (event) => {
  state.search = event.target.value;
  render();
});

loadDocs().catch((error) => {
  const detail = document.getElementById("detail-view");
  document.getElementById("detail-empty").classList.add("hidden");
  detail.innerHTML = `<article class="card"><h2>Failed to load docs</h2><p>${error}</p></article>`;
});
