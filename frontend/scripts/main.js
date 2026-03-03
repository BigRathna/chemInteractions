import { API } from "./api.js";
import { UI } from "./ui.js";

document.addEventListener("DOMContentLoaded", () => {
  const addReactantBtn = document.getElementById("add-reactant");
  const reactantsContainer = document.getElementById("reactants-container");
  const predictBtn = document.getElementById("predict-btn");
  const predictBtnNl = document.getElementById("predict-btn-nl");
  const nlTextarea = document.getElementById("nl-textarea");

  const modeSciBtn = document.getElementById("mode-scientific");
  const modeNlBtn = document.getElementById("mode-nl");
  const sciSection = document.getElementById("scientific-input");
  const nlSection = document.getElementById("nl-input");

  let reactantCount = 1;

  // Toggle Logic
  modeSciBtn.addEventListener("click", () => {
    modeSciBtn.classList.add("active");
    modeNlBtn.classList.remove("active");
    sciSection.style.display = "block";
    nlSection.style.display = "none";
  });

  modeNlBtn.addEventListener("click", () => {
    modeNlBtn.classList.add("active");
    modeSciBtn.classList.remove("active");
    nlSection.style.display = "block";
    sciSection.style.display = "none";
  });

  addReactantBtn.addEventListener("click", () => {
    reactantCount++;
    const div = document.createElement("div");
    div.className = "input-container fade-in";
    div.innerHTML = `
            <label class="mono" style="font-size: 0.75rem;">REACTANT_0${reactantCount}</label>
            <input type="text" placeholder="Enter compound name or SMILES..." class="reactant-input">
        `;
    reactantsContainer.appendChild(div);
  });

  const runPrediction = async (reactants, conditions) => {
    UI.setLoading(true);
    UI.clearResults();

    try {
      const result = await API.predict(reactants, conditions);
      UI.renderReaction(result);
    } catch (error) {
      console.error(error);
      alert("PIPELINE_FAILURE: SEE_CONSOLE");
    } finally {
      UI.setLoading(false);
    }
  };

  predictBtn.addEventListener("click", async () => {
    const inputs = document.querySelectorAll(".reactant-input");
    const reactants = Array.from(inputs)
      .map((i) => i.value)
      .filter((v) => v.trim() !== "");

    if (reactants.length === 0) {
      alert("SYSTEM_ERROR: INPUT_REQUIRED");
      return;
    }

    const temperature =
      parseFloat(document.getElementById("temp-input").value) || null;
    const catalyst = document.getElementById("catalyst-input").value || null;

    const conditions = {
      temperature,
      catalyst,
      ph: null,
      raw_input: null,
    };

    await runPrediction(reactants, conditions);
  });

  predictBtnNl.addEventListener("click", async () => {
    const text = nlTextarea.value.trim();
    if (!text) {
      alert("SYSTEM_ERROR: DESCRIPTION_REQUIRED");
      return;
    }

    // In Natural Language mode, we treat the whole text as raw input
    // The backend will handle parsing it into reactants and conditions (heuristic based for now)
    await runPrediction([], text);
  });
});
