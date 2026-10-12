/** Soft parallax on the hero media for presence without distraction. */
const media = document.querySelector('.hero-media');
if (media && !window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
  window.addEventListener(
    'scroll',
    () => {
      const y = Math.min(window.scrollY, 480);
      media.style.transform = `translateY(${y * 0.18}px) scale(${1 + y / 8000})`;
    },
    { passive: true },
  );
}
