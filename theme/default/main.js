const compact = window.matchMedia('(max-width: 1100px)');
const buttons = [...document.querySelectorAll('[data-toggle]')];
const preferences = {};
for (const name of ['tree', 'toc']) {
  try { preferences[name] = localStorage.getItem(`panel-${name}`); } catch {}
}
function show(name, expanded) {
  document.body.classList.toggle(`${name}-collapsed`, !expanded);
  document.querySelector(`[data-toggle="${name}"]`).setAttribute('aria-expanded', String(expanded));
}
function layout() {
  for (const name of ['tree', 'toc']) show(name, !compact.matches && preferences[name] !== 'closed');
}
for (const button of buttons) button.addEventListener('click', () => {
  const name = button.dataset.toggle;
  const expanded = button.getAttribute('aria-expanded') !== 'true';
  if (compact.matches) {
    show(name === 'tree' ? 'toc' : 'tree', false);
  } else {
    preferences[name] = expanded ? 'open' : 'closed';
    try { localStorage.setItem(`panel-${name}`, preferences[name]); } catch {}
  }
  show(name, expanded);
});
document.querySelector('#toc').addEventListener('click', event => {
  if (compact.matches && event.target.closest('a')) show('toc', false);
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && compact.matches) { show('tree', false); show('toc', false); }
});
compact.addEventListener('change', layout);
layout();
