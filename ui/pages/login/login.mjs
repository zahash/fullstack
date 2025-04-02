const ele_username = document.getElementById("login-username");
const ele_password = document.getElementById("login-password");
const ele_remember = document.getElementById("login-remember");

/**
 * Handles the login form submission.
 *
 * @param {SubmitEvent} event - The event object from the form submission.
 */
export default async function login(event) {
    event.preventDefault();

    const response = await fetch("/login", {
        method: "POST",
        credentials: "include",
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            "username": ele_username.value,
            "password": ele_password.value,
            "remember": ele_remember.checked ? "true" : "false"
        })
    });

    if (response.ok) {
        console.log("login successful");
    } else {
        console.log(await response.json());
    }
}

document.getElementById("login-form").addEventListener("submit", login);
