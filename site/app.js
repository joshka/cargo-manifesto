(() => {
  "use strict";

  // The theme control works even if the demonstration data cannot be read.
  const themeButtons = [...document.querySelectorAll("[data-theme-choice]")];
  function showTheme() {
    const current = document.documentElement.dataset.theme ?? "system";
    themeButtons.forEach((button) => {
      button.setAttribute("aria-pressed", String(button.dataset.themeChoice === current));
    });
  }
  themeButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const choice = button.dataset.themeChoice;
      if (choice === "system") delete document.documentElement.dataset.theme;
      else document.documentElement.dataset.theme = choice;
      try {
        if (choice === "system") localStorage.removeItem("theme");
        else localStorage.setItem("theme", choice);
      } catch {
        // Private browsing may refuse storage; the choice still applies to this page.
      }
      showTheme();
    });
  });
  showTheme();
  document.querySelector(".theme").hidden = false;

  let data;
  try {
    data = JSON.parse(document.getElementById("doctrine-data").textContent);
  } catch {
    // The static manifest and articles remain readable without the interactive register.
    return;
  }
  const { doctrines, amendments, latestEdition } = data;
  const announcement = document.getElementById("announcement");
  const state = { dependencies: "none", versions: "exact", panic: "unwind", edition: "settled" };
  let count = 0;

  function manifest() {
    const lines = [
      "[package]",
      'name = "enduring-convictions"',
      'version = "0.1.0"',
      `edition = "${state.edition === "current" ? latestEdition : "2021"}"`,
      "",
    ];
    if (state.dependencies === "serde") {
      const requirement = state.versions === "exact" ? "=1.0.228" : "1";
      lines.push("[dependencies]", `serde = "${requirement}"`, "");
    }
    lines.push("[profile.release]", `panic = "${state.panic}"`);
    return lines.join("\n");
  }

  // The rule each topic currently supports, or none when the topic has no evidence.
  function rule(topic, from = state) {
    switch (topic) {
      case "dependencies":
        return from.dependencies === "none" ? "dependencies.independence" : "dependencies.cooperation";
      case "versions":
        return from.dependencies === "none" ? null : `versions.${from.versions}`;
      case "panic":
        return `panic.${from.panic}`;
      case "edition":
        return `edition.${from.edition}`;
    }
  }
  const topics = ["edition", "dependencies", "versions", "panic"];

  function element(tag, text, className) {
    const node = document.createElement(tag);
    if (text !== undefined) node.textContent = text;
    if (className) node.className = className;
    return node;
  }

  function renderManifesto() {
    document.getElementById("manifest-code").textContent = manifest();
    const articles = document.getElementById("manifesto-articles");
    articles.replaceChildren(
      ...topics.map((topic) => rule(topic)).filter(Boolean).map((id) => {
        const article = element("article");
        article.append(element("h3", doctrines[id].title), element("p", doctrines[id].text));
        return article;
      }),
    );
    // Keep the controls in step with the manifest; with no dependency, no requirement applies.
    document.querySelector(`input[name="dependencies"][value="${state.dependencies}"]`).checked = true;
    document.querySelectorAll('input[name="versions"]').forEach((input) => {
      input.checked = state.dependencies === "serde" && input.value === state.versions;
    });
  }

  function record(topic, before, after) {
    const amendment = amendments[topic];
    count += 1;
    const entry = element("li");
    entry.append(
      element("h3", `Amendment ${count}: ${amendment.title}`),
      element("p", "Previously, we held:", "register-label"),
      element("p", doctrines[before].text),
      element("p", "We now hold:", "register-label"),
      element("p", doctrines[after].text),
      element("p", amendments.continuity.replace("{subject}", amendment.subject), "register-continuity"),
    );
    const register = document.getElementById("register");
    register.prepend(entry);
    document.getElementById("register-empty").hidden = true;
    document.getElementById("amendment-count").textContent = String(count);
    announcement.textContent = `Amendment ${count} recorded: ${amendment.title}. Positions changed: 0.`;
  }

  document.querySelector(".controls").addEventListener("change", (event) => {
    const input = event.target;
    const previous = { ...state };
    state[input.name] = input.value;
    // Choosing a requirement only makes sense for a declared dependency, so declare it.
    if (input.name === "versions") state.dependencies = "serde";
    renderManifesto();
    // Like --amend, only a doctrine that switches sides is recorded. A doctrine that merely
    // appears or disappears with its evidence is not an amendment.
    for (const topic of topics) {
      const before = rule(topic, previous);
      const after = rule(topic);
      if (before && after && before !== after) record(topic, before, after);
    }
  });

  document.querySelectorAll("[data-copy]").forEach((block) => {
    const button = block.querySelector(".copy");
    const code = block.querySelector("code");
    let reset;
    button.addEventListener("click", async () => {
      clearTimeout(reset);
      try {
        await navigator.clipboard.writeText(code.textContent);
        button.textContent = "Copied";
        announcement.textContent = "Copied to the clipboard.";
      } catch {
        // Select the text so the browser's own copy command still works.
        const range = document.createRange();
        range.selectNodeContents(code);
        getSelection().removeAllRanges();
        getSelection().addRange(range);
        button.textContent = "Selected";
        announcement.textContent = "Commands selected. Use your browser’s copy command.";
      }
      reset = setTimeout(() => { button.textContent = "Copy"; }, 2400);
    });
  });

  renderManifesto();
  document.querySelectorAll("[data-interactive]").forEach((node) => { node.hidden = false; });
})();
