import initRouter from "./lib/router.mjs";

initRouter({
    "/": { template: "./pages/home/home.html", title: "Home", scripts: ["./pages/home/home.js"] },
    "/login": { template: "./pages/login/login.html", title: "Login", scripts: ["./pages/login/login.js"] },
    "/signup": { template: "./pages/signup/signup.html", title: "Sign Up", scripts: ["./pages/signup/signup.mjs"] }
});
