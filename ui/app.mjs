import initRouter from "./lib/router.mjs";

export const hooks = initRouter({
    "/": { template: "./pages/home/home.html", title: "Home", scripts: ["./pages/home/home.js"] },
    "/login": { template: "./pages/login/login.html", title: "Login", scripts: ["./pages/login/login.mjs"] },
    "/signup": { template: "./pages/signup/signup.html", title: "Sign Up", scripts: ["./pages/signup/signup.mjs", "./pages/signup/lifecycle.js"] }
}, {
    fragments: {
        header: "./fragments/header.html",
        nav: "./fragments/nav.html",
    }
});
