// The header follows the scroll, as in X: scrolling a screen's content down
// slides its header up into the status bar strip, point for point, its
// contents fading into the bar's background as it goes until it is gone,
// and any scroll up brings it back the same way. At the top of the content
// it is always fully shown. One listener, in the capture phase since scroll
// events do not bubble, serves every screen. It guards itself so a second
// load does nothing.
(() => {
    if (window.__cairnHeaderHide) return;
    window.__cairnHeaderHide = true;

    // The last scroll position seen per scroll area, and how far each header is
    // pushed up, in pixels.
    const lastTop = new WeakMap();
    const offset = new WeakMap();

    document.addEventListener(
        "scroll",
        (event) => {
            const area = event.target;
            if (!(area instanceof Element) || !area.classList.contains("screen-content")) return;
            const nav = area.parentElement && area.parentElement.querySelector(":scope > .app-nav");
            if (!nav) return;
            // The menu is open: leave the header where it is.
            if (nav.querySelector(".nav-panel-open")) return;

            const top = Math.max(0, area.scrollTop);
            const bottom = area.scrollHeight - area.clientHeight;
            const previous = lastTop.has(area) ? lastTop.get(area) : top;
            lastTop.set(area, top);
            // The bounce past the bottom edge, and its spring back, are not
            // scrolling: at the bottom the header may only go, never come back.
            if (top > bottom || previous > bottom) return;
            if (top < previous && top >= bottom - 2) return;

            const height = nav.offsetHeight;
            let pushed = (offset.get(nav) || 0) + (top - previous);
            pushed = top <= 0 ? 0 : Math.min(height, Math.max(0, pushed));
            offset.set(nav, pushed);
            nav.style.transform = pushed ? `translateY(${-pushed}px)` : "";
            nav.style.setProperty("--nav-fade", String(1 - pushed / height));
            nav.parentElement.classList.toggle("header-collapsed", pushed >= height);
        },
        { capture: true, passive: true },
    );

    // The header is shared across screens, so a new screen inherits the push
    // the last one left it with, and its content at the top sends no scroll
    // event to undo it. A mount that brings a fresh scroll area at its top
    // shows the header in full.
    let lastArea = null;
    const observer = new MutationObserver(() => {
        for (const area of document.querySelectorAll(".screen-content")) {
            if (area === lastArea) continue;
            lastArea = area;
            const nav = area.parentElement && area.parentElement.querySelector(":scope > .app-nav");
            if (!nav || area.scrollTop > 0) continue;
            offset.set(nav, 0);
            nav.style.transform = "";
            nav.style.removeProperty("--nav-fade");
            nav.parentElement.classList.remove("header-collapsed");
        }
    });
    observer.observe(document.documentElement, { childList: true, subtree: true });
})();
