import '@testing-library/jest-dom/vitest';

const APP_VERSION_META = 'app-version';

if (typeof document !== 'undefined') {
  if (!document.querySelector(`meta[name="${APP_VERSION_META}"]`)) {
    const meta = document.createElement('meta');
    meta.setAttribute('name', APP_VERSION_META);
    meta.setAttribute('content', 'test');
    document.head.appendChild(meta);
  }
}

if (typeof window !== 'undefined' && typeof window.matchMedia !== 'function') {
  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    writable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => undefined,
      removeListener: () => undefined,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      dispatchEvent: () => false,
    }),
  });
}
