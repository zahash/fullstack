/**
 * @typedef {Object} Route
 * @property {string} template - The path to the HTML file for the route.
 */

/**
 * Initializes a client-side hash-based router.
 * Supports route-level lifecycle hooks (onMount/onUnmount), dynamic fragment resolution,
 * script/style injection, prefetching, and internal link rewriting.
 *
 * @param {Object} options
 * @param {Object} options.routes - A map of path => { template: string }.
 * @param {Object} options.fragments - A map of fragment name => URL for dynamic HTML fragments.
 * @returns {Promise<{
*   redirect: (href: string) => void,
*   onMount: (fn: () => void | Promise<void>) => void,
*   onUnmount: (fn: () => void | Promise<void>) => void,
*   ready: () => void
* }>}
*
* @example
* import initRouter from "./lib/router.mjs";
*
* export const hooks = await initRouter({
*   routes: {
*     "/": { template: "home.html" },
*     "/login": { template: "./pages/login.html" },
*     "/signup": { template: "./pages/signup.html" }
*   },
*   fragments: {
*     header: "./fragments/header.html",
*     nav: "./fragments/nav.html"
*   }
* });
*/
export default async function initRouter({ routes = {}, fragments = {} }) {

    let lifecycle = {
        onMount: {},
        onUnmount: {},
    };

    /**
     * Handles navigation when a user clicks a link.
     * Updates the URL hash without triggering a full page reload.
     *
     * @param {MouseEvent} event - The click event from the navigation link.
     */
    function route(event) {
        const anchor = event.target.closest("a[href]");
        if (!anchor) return;

        const href = anchor.getAttribute("href");
        if (href.startsWith("#")) {
            event.preventDefault();
            window.location.hash = href; // hashchange triggers render
        }
    }

    /**
     * Resolves all nested fragments within an HTML string before rendering.
     * This prevents layout shifts by fully resolving fragments before insertion.
     * Also handles dynamic script and style loading.
     *
     * @param {string} html - Raw HTML string of the page template.
     * @returns {Promise<DocumentFragment>} - Fully resolved DOM fragment ready for insertion.
     */
    async function resolvePage(html) {
        /**
         * Updates document.title based on a <meta data-title> tag in the page.
         *
         * @param {DocumentFragment} page
         */
        function resolveTitle(page) {
            const metaTitle = page.querySelector('meta[data-title]');
            document.title = metaTitle?.getAttribute("data-title") || "Untitled Page";
            metaTitle?.remove();
        }

        /**
         * Recursively resolves <div data-fragment="name"> with associated fragments.
         *
         * @param {DocumentFragment} fragment - DOM fragment to scan and mutate.
         * @returns {Promise<void>}
         */
        async function resolveNestedFragments(fragment) {
            const slots = Array.from(fragment.querySelectorAll("div[data-fragment]"));
            if (!slots.length) return;

            await Promise.all(slots.map(async slot => {
                const slotName = slot.getAttribute("data-fragment");
                const fragmentPath = fragments[slotName];
                if (!fragmentPath) return;

                try {
                    const response = await fetch(fragmentPath);
                    if (!response.ok) {
                        console.error(`failed to load fragment { ${slotName} : "${fragmentPath}" }`);
                        return;
                    }
                    const html = await response.text();
                    const innerFragment = document.createRange().createContextualFragment(html);
                    await resolveNestedFragments(innerFragment);
                    slot.replaceWith(innerFragment);
                } catch (err) {
                    console.error(`failed to fetch fragment { ${slotName} : "${fragmentPath}" }`);
                    console.error(err);
                }
            }));
        }

        /**
         * Loads <link> and <style> tags from the page and injects them into <head>.
         *
         * @param {DocumentFragment} page
         */
        function resolveStyles(page) {
            document.querySelectorAll("link[data-dynamic-style]").forEach(link => link.remove());
            (page.querySelectorAll('link[rel="stylesheet"][href]') || []).forEach(link => {
                link.setAttribute("data-dynamic-style", ""); // Mark as dynamically loaded
                document.head.appendChild(link);
            });

            document.querySelectorAll("style[data-dynamic-style]").forEach(style => style.remove());
            (page.querySelectorAll("style") || []).forEach(style => {
                style.setAttribute("data-dynamic-style", ""); // Mark as dynamically loaded
                document.head.appendChild(style);
            });
        }

        /**
         * Loads <script> tags from the page and injects them into <body>.
         *
         * @param {DocumentFragment} page
         */
        function resolveScripts(page) {
            document.querySelectorAll("script[data-dynamic-script]").forEach(script => script.remove());
            (page.querySelectorAll("script") || []).forEach(script => {
                script.setAttribute("data-dynamic-script", "");
                document.body.appendChild(script);
            });
        }

        const page = document.createRange().createContextualFragment(html);

        resolveTitle(page);
        await resolveNestedFragments(page);
        resolveStyles(page);
        resolveScripts(page);

        return page;
    }

    /**
     * Normalizes a hash value to a clean path.
     *
     * @param {string} hash
     * @returns {string | null}
     */
    function dehash(hash) {
        hash = hash || "/";
        if (hash.startsWith("#")) return hash.slice(1) || "/";
        if (hash.startsWith("/")) return hash;
        console.warn("cannot dehash:", hash);
        return null;
    }

    /**
     * Loads the appropriate content based on the current URL hash.
     * Fetches the corresponding HTML file, resolves fragments, and injects it into the DOM.
     *
     * @returns {Promise<void>}
     */
    async function render() {
        const root = document.getElementById("root");

        const path = dehash(window.location.hash);
        const route = routes[path] || routes[404];
        if (!route) {
            root.innerHTML = "<h1>404 - Page Not Found</h1>";
            document.title = "404 - Page Not Found";
            return;
        }

        const response = await fetch(route.template);
        const html = await response.text();
        const page = await resolvePage(html);
        root.replaceChildren(page);

        // Convert all internal `<a>` links (starting with `/`) to hash-based routes.
        document.querySelectorAll("a[href]").forEach(anchor => {
            const href = anchor.getAttribute("href");
            if (href.startsWith("/")) {
                anchor.setAttribute("href", `#${href}`);
            }
        });
    }

    /**
     * Prefetches content when hovering over a link with `data-prefetch`.
     *
     * @param {MouseEvent | TouchEvent} event - The event triggered by hovering or touching a link.
     */
    function prefetch(event) {
        const anchor = event.target.closest("a[data-prefetch][href]");
        if (!anchor) return;

        const href = anchor.getAttribute("href");
        if (!href.startsWith("#")) return;

        const path = href.slice(1) || "/"; // Remove "#" and default to "/"
        const route = routes[path] || routes[404];

        if (route) fetch(route.template);
    }

    document.addEventListener("click", route); // Listen for clicks on the entire document
    document.addEventListener("mouseover", prefetch);
    document.addEventListener("touchstart", prefetch); // `touchstart` for Mobile support
    window.addEventListener("hashchange", async event => { // Handle back/forward navigation using hashchange
        const from = dehash(new URL(event.oldURL).hash);
        const to = dehash(new URL(event.newURL).hash);

        await (lifecycle.onUnmount[from] ?? (async () => { }))();
        await render();
        await (lifecycle.onMount[to] ?? (async () => { }))();
    });
    window.addEventListener("module:ready", async event => {
        await (lifecycle.onMount[event.detail.route] ?? (async () => { }))();
    });

    await render(); // initial render on first page load

    return {
        redirect: (href) => window.location.hash = href,
        onMount: (fn) => lifecycle.onMount[dehash(window.location.hash)] = fn,
        onUnmount: (fn) => lifecycle.onUnmount[dehash(window.location.hash)] = fn,
        ready: () => window.dispatchEvent(new CustomEvent("module:ready", { detail: { route: dehash(window.location.hash) } })),
    }
}
