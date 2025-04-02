import initRouter from "./lib/router.mjs";

initRouter({
    "/": { template: "./pages/home/home.html", title: "Home", scripts: ["./pages/home/home.mjs"] },
    "/login": { template: "./pages/login/login.html", title: "Login", scripts: ["./pages/login/login.mjs"] },
    "/signup": { template: "./pages/signup/signup.html", title: "Sign Up", scripts: ["./pages/signup/signup.mjs"] }
});
