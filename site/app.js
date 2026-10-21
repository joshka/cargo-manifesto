(() => {
  "use strict";

  const dataElement = document.getElementById("doctrine-data");
  let data;
  try {
    data = JSON.parse(dataElement.textContent);
  } catch {
    // The first doctrine remains readable even when the interactive edition is unavailable.
    return;
  }

  const scenarios = new Map(data.scenarios.map((scenario) => [scenario.id, scenario]));
  const scenarioButtons = [...document.querySelectorAll("[data-scenario]")];
  const choices = [...document.querySelectorAll('input[name="implementation"]')];
  const title = document.getElementById("doctrine-title");
  const doctrine = document.getElementById("doctrine-text");
  const code = document.getElementById("manifest-code");
  const panel = document.querySelector(".doctrine-panel");
  const announcement = document.getElementById("demo-announcement");
  let currentScenario = "dependencies";
  let currentChoice = 0;

  function render(announce = true) {
    const scenario = scenarios.get(currentScenario);
    const choice = scenario.choices[currentChoice];
    title.textContent = choice.title;
    doctrine.textContent = choice.text;
    code.textContent = choice.code;
    scenario.choices.forEach((option, index) => {
      document.getElementById(`choice-${index}-label`).textContent = option.label;
      choices[index].checked = index === currentChoice;
    });
    scenarioButtons.forEach((button) => {
      const selected = button.dataset.scenario === currentScenario;
      button.classList.toggle("is-selected", selected);
      button.setAttribute("aria-pressed", String(selected));
    });
    if (announce) {
      announcement.textContent = `${choice.label}. ${choice.title}. Principles upheld. These principles have guided this project from the beginning.`;
      document.getElementById("continuity-note").textContent = "The record has been clarified.";
      panel.classList.remove("is-clarified");
      // Restart a short transition when another doctrine is selected.
      void panel.offsetWidth;
      panel.classList.add("is-clarified");
    }
  }

  scenarioButtons.forEach((button) => {
    button.addEventListener("click", () => {
      currentScenario = button.dataset.scenario;
      currentChoice = 0;
      render();
    });
  });
  choices.forEach((choice) => {
    choice.addEventListener("change", () => {
      currentChoice = Number(choice.value);
      render();
    });
  });

  const copyButton = document.querySelector(".copy-button");
  const copyAnnouncement = document.getElementById("copy-announcement");
  let copyReset;
  copyButton.addEventListener("click", async () => {
    const commands = document.getElementById("install-commands").textContent;
    try {
      await navigator.clipboard.writeText(commands);
      copyButton.textContent = "Copied ✓";
      copyAnnouncement.textContent = "Installation commands copied to the clipboard.";
      clearTimeout(copyReset);
      copyReset = setTimeout(() => { copyButton.textContent = "Copy commands ⧉"; }, 2400);
    } catch {
      // Leave the exact commands selected for ordinary browser copy when clipboard access is denied.
      const range = document.createRange();
      range.selectNodeContents(document.getElementById("install-commands"));
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      clearTimeout(copyReset);
      copyButton.textContent = "Selected — copy manually";
      copyAnnouncement.textContent = "Commands selected. Use your browser’s Copy command.";
      copyReset = setTimeout(() => { copyButton.textContent = "Copy commands ⧉"; }, 5000);
    }
  });

  render(false);
  document.querySelectorAll("[data-interactive]").forEach((element) => { element.hidden = false; });
})();
