# Phase 4 — Frontend Design

## Goal

Build a clean, premium-looking web interface for inputting chemical information and visualizing prediction results.

---

## Design Philosophy

- **Modern & Minimal**: Dark mode support, glassmorphism, subtle micro-animations.
- **Scientific**: Clear typography (Inter/Roboto), probability bars, chemical structure badges.
- **Responsive**: Works on desktop and tablet.

---

## Layout Sections

### 1. Header

- Logo: "ChemInteractions" with an atom/bond icon.
- Links: Predictor (main), Reaction Library, About.

### 2. Main Input Panel (Predictor)

- **Reactants Input**: A dynamic list of inputs. "Add Reactant" button.
- **PubChem Autocomplete**: As user types, show suggestions (name + formula).
- **Conditions Panel**:
  - Optional: Temperature (Reflux/Room Temp/Slider), pH, Catalyst type, Solvent.
- **Action**: Large "Predict Interaction" button with a pulse animation.

### 3. Results Overview

- "Found X possible interactions."
- Side-by-side or stacked **Reaction Cards**.

### 4. Reaction Card Component

- **Title**: e.g., "Fischer Esterification".
- **Probability Badge**: Visual bar + %.
- **Confidence Tier Badge**: "Verified" (Green), "ML Predicted" (Blue), etc.
- **Product Display**:
  - List of products. Primary products highlighted.
  - Each has: Name, SMILES, Formula.
- **Mechanism (Accordion)**: Markdown rendered explanation & citations.
- **Hazards**: Red warning banners if applicable.

---

## Technology

- **HTML5 / Vanilla CSS**: No heavy frameworks for the prototype.
- **Vanilla Javascript**: Modules for API client and state management.
- **Icons**: Lucide Icons or Heroicons.
- **Markdown Rendering**: `marked.js` or `showdown.js`.

---

## API Client (`frontend/scripts/api.js`)

```javascript
export async function predictInteraction(reactants, conditions) {
  const response = await fetch("/api/v1/predict", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ reactants, conditions }),
  });
  return response.json();
}

export async function resolveCompound(query) {
  const resp = await fetch(
    `/api/v1/compound/resolve?q=${encodeURIComponent(query)}`,
  );
  return resp.json();
}
```

---

## Dynamic UI Flow

1. User types "Acetic Acid" → calls `resolveCompound(query)`.
2. UI displays "C2H4O2 (Found)" below the input.
3. User adds "Ethanol" and selects "Reflux" + "Acid Catalyst".
4. User clicks "Predict" → button shows loading spinner.
5. API returns JSON → UI clears results area and renders Cards.
6. User clicks a Card → Accordion opens showing the Mechanism.

---

## Styling (CSS Palette)

- **Background**: `#0f172a` (Deep Slate)
- **Cards**: Surface with backdrop-filter blur + `#1e293b`.
- **Primary Accent**: `#38bdf8` (Light Blue)
- **Success/Verified**: `#4ade80` (Green)
- **Warning**: `#fbbf24` (Amber)
- **Hazard**: `#f87171` (Red)

---

## Checklist

- [ ] Responsive grid layout for cards.
- [ ] Loading state (skeletons or spinner) during prediction.
- [ ] Markdown mechanisms render correctly with citations.
- [ ] Autocomplete makes input feel "smart" and scientific.
- [ ] Copy to clipboard button for SMILES/JSON.
      v
