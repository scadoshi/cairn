// The entrance, run inside the page: a card arrives as the viewport reaches
// it, and the figures in it roll their digits into place. It lives here
// rather than on the app side because iOS holds the app's messages to the
// page until a scroll settles, so anything decided over there lands late.
// Here, IntersectionObserver and requestAnimationFrame keep running while
// the page moves.
(function () {
    var reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
    var ROLL_MS = 1000;

    function textNode(el) {
        for (var i = 0; i < el.childNodes.length; i++) {
            var n = el.childNodes[i];
            if (n.nodeType === 3 && n.data.trim()) return n;
        }
        return null;
    }

    // Rolls one figure: every digit flickers through random ones with a
    // chance that falls to nothing, squared so it is wild early and barely
    // moves at the end. The rest of the text stays put, so the width holds.
    // Writes go into the text node the app rendered, so the app's own next
    // write is to the same node; the first write that is not ours ends ours.
    function roll(el) {
        var t = textNode(el);
        if (!t || el.dataset.rolling) return;
        var real = t.data, last = real, start = null;
        el.dataset.rolling = "1";
        function done() { setTimeout(function () { delete el.dataset.rolling; }, 0); }
        function frame(now) {
            if (t.data !== last) { done(); return; }
            if (start === null) start = now;
            var f = Math.min(1, (now - start) / ROLL_MS);
            if (f >= 1) { t.data = real; done(); return; }
            var spread = (1 - f) * (1 - f), out = "";
            for (var i = 0; i < real.length; i++) {
                var c = real[i];
                out += (c >= "0" && c <= "9" && Math.random() < spread) ? String(Math.floor(Math.random() * 10)) : c;
            }
            t.data = out; last = out;
            requestAnimationFrame(frame);
        }
        requestAnimationFrame(frame);
    }

    function reveal(el) {
        el.classList.remove("reveal-pending");
        el.classList.add("reveal-in");
        if (!reduce) el.querySelectorAll(".figure").forEach(roll);
    }

    // A card starts a little before it is on screen, so the entrance reads
    // as it arrives rather than after.
    var io = new IntersectionObserver(function (entries) {
        entries.forEach(function (e) {
            if (e.isIntersecting) { io.unobserve(e.target); reveal(e.target); }
        });
    }, { rootMargin: "0px 0px 30% 0px" });

    function scan(root) {
        var list = root.querySelectorAll ? root.querySelectorAll(".reveal:not([data-seen])") : [];
        list.forEach(function (el) {
            el.dataset.seen = "1";
            if (reduce) { el.classList.add("reveal-in"); return; }
            el.classList.add("reveal-pending");
            io.observe(el);
        });
    }

    // New cards (a screen change) get observed, and a figure added to a card
    // that has already arrived (Home's tiles, drawn once the counters load)
    // rolls in. A figure the app rewrites in place (a tap) does not: the
    // number just changes, since a roll on every tap read as broken.
    new MutationObserver(function (muts) {
        var added = [];
        muts.forEach(function (m) {
            m.addedNodes.forEach(function (n) {
                if (n.nodeType !== 1) return;
                scan(n);
                if (!n.closest(".reveal-in")) return;
                if (n.classList.contains("figure")) added.push(n);
                n.querySelectorAll(".figure").forEach(function (f) { added.push(f); });
            });
        });
        if (reduce) return;
        added.forEach(function (el) {
            var card = el.closest(".reveal");
            if (!el.dataset.rolling && (!card || card.classList.contains("reveal-in"))) roll(el);
        });
    }).observe(document.body, { subtree: true, childList: true });

    scan(document);
})();
