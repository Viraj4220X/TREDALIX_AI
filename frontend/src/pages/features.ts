import { featureBlockedReason, featureRegistry } from '../settings/features.ts';
import { escapeHtml } from '../components/layout.ts';
import type { AppSettings } from '../types.ts';

export function renderFeatures(settings: AppSettings): string {
  const cards = featureRegistry
    .map((feature) => {
      const blocked = featureBlockedReason(feature.id, settings.features);
      const configurable =
        feature.status === 'available' || feature.status === 'experimental';
      const enabled = settings.features[feature.id]?.enabled ?? false;
      const cannotEnable = Boolean(blocked && !enabled);
      return `<article class="feature-card ${configurable ? '' : 'feature-muted'}">
        <div class="feature-card-top"><div><span class="status-pill status-${feature.status}">${escapeHtml(feature.status)}</span><h2>${escapeHtml(feature.name)}</h2></div>
          ${configurable ? `<label class="switch" aria-label="${escapeHtml(feature.name)} ${enabled ? 'enabled' : 'disabled'}"><input type="checkbox" data-feature="${escapeHtml(feature.id)}" ${enabled ? 'checked' : ''} ${cannotEnable ? 'disabled' : ''} /><span class="switch-track"></span></label>` : '<span class="lock-mark" aria-label="Unavailable">LOCKED</span>'}
        </div>
        <p>${escapeHtml(feature.description)}</p>
        <div class="feature-card-footer"><span>${feature.platforms.length ? escapeHtml(feature.platforms.join(' · ')) : 'No supported platform yet'}</span>${blocked ? `<small>${escapeHtml(blocked)}</small>` : '<small>Ready when enabled</small>'}</div>
      </article>`;
    })
    .join('');
  return `<section class="feature-summary panel"><div class="feature-summary-icon">FX</div><div><strong>Availability is separate from preference.</strong><p>Unavailable features cannot be enabled. Disabling a feature blocks the corresponding application operation; it does not delete local data.</p></div></section>
    <section class="feature-grid">${cards}</section>`;
}
