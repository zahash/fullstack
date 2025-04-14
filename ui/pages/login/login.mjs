import { hooks } from "../../app.mjs";

/**
 * Handles the login form submission.
 *
 * @param {SubmitEvent} event - The event object from the form submission.
 */
async function login(event) {
    event.preventDefault();

    const response = await fetch("/login", {
        method: "POST",
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            "username": document.getElementById("login-username").value,
            "password": document.getElementById("login-password").value,
        })
    });

    if (response.ok) {
        alert("login successful");
        hooks.redirect("/");
    } else {
        alert("login failed");
    }
}

hooks.onMount(() => {
    console.log("LOGIN MOUNT");
    window.login = login;
});
hooks.onUnmount(() => {
    console.log("LOGIN UN-MOUNT");
    delete window.login;
});
hooks.ready();
