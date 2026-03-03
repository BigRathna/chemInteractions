export const API = {
    async predict(reactants, conditions) {
        const response = await fetch('/api/predict', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ reactants, conditions })
        });
        if (!response.ok) throw new Error('Prediction request failed');
        return await response.json();
    },

    async resolveCompound(query) {
        const response = await fetch(`/api/compounds?q=${encodeURIComponent(query)}`);
        if (!response.ok) throw new Error('Compound resolution failed');
        return await response.json();
    }
};
