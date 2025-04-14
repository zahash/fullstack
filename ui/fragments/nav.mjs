import {hooks} from "../app.mjs";

async function logout() {
    const response = await fetch("/logout");
    if (response.ok) {
        alert("logout successful");
        hooks.redirect("/login");
    } else {
        alert("logout failed");
    }
}

hooks.onMount(() => {
    window.logout = logout;
});
hooks.onUnmount(() => {
    delete window.logout;
});
hooks.ready();
