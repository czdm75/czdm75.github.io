const compact = window.matchMedia('(max-width: 1100px)');
// Preserve folder choices across page navigation, without persisting untouched defaults.
const treePanel = document.getElementById('tree-panel');
if (treePanel) {
  const storageKey = `site-tree-folders:v1:${new URL(treePanel.dataset.treeRoot, document.baseURI).href}`;
  let stored = {};
  try {
    const value = JSON.parse(sessionStorage.getItem(storageKey));
    if (value && typeof value === 'object' && !Array.isArray(value)) stored = value;
  } catch {}
  const folders = [...treePanel.querySelectorAll('.tree-folder[data-tree-key] > details')];
  const states = new Map();
  for (const folder of folders) {
    const key = folder.parentElement.dataset.treeKey;
    if (Object.hasOwn(stored, key) && typeof stored[key] === 'boolean') folder.open = stored[key];
    states.set(folder, folder.open);
  }
  function save() {
    for (const folder of folders) {
      if (folder.open !== states.get(folder)) {
        stored[folder.parentElement.dataset.treeKey] = folder.open;
        states.set(folder, folder.open);
      }
    }
    try { sessionStorage.setItem(storageKey, JSON.stringify(stored)); } catch {}
  }
  for (const folder of folders) folder.addEventListener('toggle', save);
  // Capture rapid expand-and-navigate clicks before the asynchronous toggle event.
  treePanel.addEventListener('click', event => {
    if (event.target.closest('a')) save();
  });
  window.addEventListener('pagehide', save);
}
const buttons = [...document.querySelectorAll('[data-toggle]')];
const names = ['tree', 'toc'].filter(name => buttons.some(button => button.dataset.toggle === name));
const preferences = {};
for (const name of names) {
  try { preferences[name] = localStorage.getItem(`panel-${name}`); } catch {}
}
function show(name, expanded) {
  if (!names.includes(name)) return;
  document.body.classList.toggle(`${name}-collapsed`, !expanded);
  for (const button of buttons.filter(button => button.dataset.toggle === name)) {
    button.setAttribute('aria-expanded', String(expanded));
  }
  document.getElementById(`${name}-content`).inert = !expanded;
}
function layout() {
  for (const name of names) show(name, !compact.matches && preferences[name] !== 'closed');
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
for (const outline of document.querySelectorAll('[data-outline]')) {
  outline.addEventListener('click', event => {
    if (compact.matches && event.target.closest('a')) {
      show(outline.closest('#tree-panel') ? 'tree' : 'toc', false);
    }
  });
}
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && compact.matches) { show('tree', false); show('toc', false); }
});
compact.addEventListener('change', layout);
layout();
