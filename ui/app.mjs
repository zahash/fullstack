import initRouter from "./lib/router.mjs";

initRouter({
    "/": { template: "./home.html", title: "Home", scripts: ["./home.mjs"] },
    "/login": { template: "./login.html", title: "Login", scripts: ["./login.mjs"] },
    "/signup": { template: "./signup.html", title: "Sign Up", scripts: ["./signup.mjs"] }
});
