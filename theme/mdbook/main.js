// Palette controls for the adapted shell. No upstream content/editor scripts.
(() => {
  const palettes = ['light', 'rust', 'coal', 'navy', 'ayu'];
  const media = matchMedia('(prefers-color-scheme: dark)');
  const picker = document.querySelector('.theme-picker');
  const buttons = [...document.querySelectorAll('[data-theme]')];
  let preference = 'auto';
  try { preference = localStorage.getItem('mdbook-theme') || 'auto'; } catch {}
  if (!palettes.includes(preference)) preference = 'auto';
  function apply() {
    const theme = preference === 'auto' ? (media.matches ? 'navy' : 'light') : preference;
    document.documentElement.classList.remove(...palettes);
    document.documentElement.classList.add(theme);
    for (const button of buttons) button.setAttribute('aria-pressed', String(button.dataset.theme === preference));
  }
  for (const button of buttons) button.addEventListener('click', () => {
    preference = button.dataset.theme;
    try {
      if (preference === 'auto') localStorage.removeItem('mdbook-theme');
      else localStorage.setItem('mdbook-theme', preference);
    } catch {}
    apply();
    picker.open = false;
    picker.querySelector('summary').focus();
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && picker.open) {
      picker.open = false;
      picker.querySelector('summary').focus();
    }
  });
  document.addEventListener('click', event => {
    if (!picker.contains(event.target)) picker.open = false;
  });
  media.addEventListener('change', apply);
  apply();
})();
