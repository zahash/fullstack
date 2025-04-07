import initRouter from "./lib/router.mjs";

export const hooks = await initRouter({
    routes: {
        "/": { template: "./pages/home/home.html" },
        "/login": { template: "./pages/login/login.html" },
        "/signup": { template: "./pages/signup/signup.html" }
    },
    fragments: {
        header: "./fragments/header.html",
        nav: "./fragments/nav.html",
    }
});
