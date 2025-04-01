// import signal from "https://cdn.jsdelivr.net/gh/zahash/spamf@cb4b5340e379f037b19fedd5e83c2a0667142595/signal.min.js";
import initRouter from "https://cdn.jsdelivr.net/gh/zahash/spamf@cb4b5340e379f037b19fedd5e83c2a0667142595/router.min.js";

initRouter({
    "/": { template: "./home.html", title: "Home", scripts: ["./home.js"] },
    "/login": { template: "./login.html", title: "Login", scripts: ["./login.js"] },
    "/signup": { template: "./signup.html", title: "Sign Up", scripts: ["./signup.mjs"] }
});
