const usernameEle = document.getElementById("login-username");
const passwordEle = document.getElementById("login-password");
const rememberEle = document.getElementById("login-remember");

/**
 * Handles the login form submission.
 *
 * @param {SubmitEvent} event - The event object from the form submission.
 */
async function login(event) {
    event.preventDefault();

    const response = await fetch("/login", {
        method: "POST",
        credentials: "include",
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            "username": usernameEle.value,
            "password": passwordEle.value,
            "remember": rememberEle.checked ? "true" : "false"
        })
    });

    if (response.ok) {
        console.log("login successful");
    } else {
        console.log(await response.json());
    }
}
