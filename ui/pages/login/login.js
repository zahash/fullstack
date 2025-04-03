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
            "remember": document.getElementById("login-remember").checked ? "true" : "false"
        })
    });

    if (response.ok) {
        console.log("login successful");
    } else {
        console.log("login failed");
    }
}
