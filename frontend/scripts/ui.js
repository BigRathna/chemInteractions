export const UI = {
    renderReaction(data) {
        const container = document.getElementById('results-section');
        const card = document.createElement('div');
        card.className = 'card reaction-card fade-in';
        
        const tierClass = data.confidence_tier.toLowerCase().replace('_', '-');
        
        card.innerHTML = `
            <span class="confidence-badge ${tierClass}">${data.confidence_tier}</span>
            <h3 class="mono" style="color: var(--accent-primary); margin-bottom: 0.5rem;">${data.reaction_name}</h3>
            <p style="font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 1.5rem;">Confidence Score: ${(data.probability * 100).toFixed(2)}%</p>
            
            <div style="margin-bottom: 1.5rem;">
                <label class="mono" style="font-size: 0.65rem; color: var(--text-muted); display: block; margin-bottom: 0.5rem;">PRIMARY_PRODUCT</label>
                <div class="mono" style="font-size: 1rem; color: var(--text-primary);">${data.products[0].name}</div>
                <div class="mono" style="font-size: 0.75rem; color: var(--text-muted);">${data.products[0].smiles}</div>
            </div>

            ${data.byproducts.length > 0 ? `
                <div style="margin-bottom: 1.5rem;">
                    <label class="mono" style="font-size: 0.65rem; color: var(--text-muted); display: block; margin-bottom: 0.5rem;">BYPRODUCTS</label>
                    <div class="mono" style="font-size: 0.75rem; color: var(--text-secondary);">
                        ${data.byproducts.map(b => `${b.name} (${b.smiles})`).join(', ')}
                    </div>
                </div>
            ` : ''}

            <div style="padding-top: 1rem; border-top: 1px solid var(--border-subtle);">
                <label class="mono" style="font-size: 0.65rem; color: var(--text-muted); display: block; margin-bottom: 0.5rem;">EXPLANATION</label>
                <p style="font-size: 0.875rem; color: var(--text-secondary); line-height: 1.4;">${data.explanation}</p>
            </div>

            <div class="process-timeline">
                <label class="mono" style="font-size: 0.65rem; color: var(--text-muted); display: block;">WORKING_PROCESS_TRACE</label>
                
                <div class="timeline-step">
                    <div class="step-marker info"></div>
                    <div class="step-content">
                        <div class="step-label">Stage 1: Reactant Analysis</div>
                        <div class="step-value mono" style="font-size: 0.7rem;">
                            Detected Groups: ${(data.reactant_groups || []).length > 0 ? data.reactant_groups.join(', ') : 'NONE_DETECTED'}
                        </div>
                    </div>
                </div>

                <div class="timeline-step">
                    <div class="step-marker info"></div>
                    <div class="step-content">
                        <div class="step-label">Stage 2: Neural Inference</div>
                        <div class="step-value mono" style="font-size: 0.7rem;">${data.ml_raw || 'No raw ML data'}</div>
                    </div>
                </div>

                <div class="timeline-step">
                    <div class="step-marker ${data.kb_match ? 'success' : ''}"></div>
                    <div class="step-content">
                        <div class="step-label">Stage 3: Rule Verification</div>
                        <div class="step-value mono" style="font-size: 0.7rem;">
                            ${data.kb_match ? `MATCH_FOUND: ${data.kb_match.name}` : 'NO_RULE_MATCHED'}
                        </div>
                    </div>
                </div>

                <div class="timeline-step">
                    <div class="step-marker success"></div>
                    <div class="step-content">
                        <div class="step-label">Stage 4: Fusion Output</div>
                        <div class="step-value mono" style="font-size: 0.7rem;">Verified Class: ${data.reaction_name}</div>
                    </div>
                </div>
            </div>
        `;
        
        container.appendChild(card);
    },

    clearResults() {
        document.getElementById('results-section').innerHTML = '';
    },

    setLoading(isLoading) {
        const btn = document.getElementById('predict-btn');
        if (isLoading) {
            btn.innerText = 'PROCESSING_PIPELINE...';
            btn.style.opacity = '0.5';
            btn.disabled = true;
        } else {
            btn.innerText = 'Execute Prediction Pipeline';
            btn.style.opacity = '1';
            btn.disabled = false;
        }
    }
};
