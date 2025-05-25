import signal, { merge } from "../../lib/signal.mjs";
import debounce from "../../debounce.mjs";
import { hooks } from "../../app.mjs";
import initWasm, { validate_password } from "../../lib/shared/wasm.js";

let usernameStatus = signal({ status: undefined, message: undefined });
let passwordStatus = signal({ status: undefined, message: undefined });
let emailStatus = signal({ status: undefined, message: undefined });

let canSignup = merge({ usernameStatus, passwordStatus, emailStatus }).derive(obj =>
    obj.usernameStatus.status === "ok" &&
    obj.passwordStatus.status === "ok" &&
    obj.emailStatus.status === "ok"
);

usernameStatus.effect(({ status, message }) => {
    const ele_msg_username = document.getElementById("signup-msg-username");

    switch (status) {
        case "unavailable":
            ele_msg_username.textContent = "username taken";
            ele_msg_username.style.display = "block";
            break;
        case "invalid":
            ele_msg_username.textContent = message || "invalid username";
            ele_msg_username.style.display = "block";
            break;
        default:
            ele_msg_username.style.display = "none";
    }
});
passwordStatus.effect(({ status, message }) => {
    const ele_msg_password = document.getElementById("signup-msg-password");

    switch (status) {
        case "weak":
            ele_msg_password.textContent = message;
            ele_msg_password.style.display = "block";
            break;
        case "ok":
            ele_msg_password.style.display = "none";
            break;
        default:
            ele_msg_password.style.display = "none";
    }
});
emailStatus.effect(({ status, message }) => {
    const ele_msg_email = document.getElementById("signup-msg-email");

    switch (status) {
        case "unavailable":
            ele_msg_email.textContent = "email taken";
            ele_msg_email.style.display = "block";
            break;
        case "invalid":
            ele_msg_email.textContent = message || "invalid email";
            ele_msg_email.style.display = "block";
            break;
        default:
            ele_msg_email.style.display = "none";
    }
});
canSignup.effect(val => document.getElementById("signup-btn").disabled = !val);

const debounced_checkUsernameAvailability = debounce(async () => {
    usernameStatus({});
    const response = await fetch(`/check/username-availability?username=${document.getElementById("signup-username").value}`);
    if (response.status == 200) usernameStatus({ status: "ok" });
    else if (response.status == 400) {
        const json = await response.json();
        const message = json["error"];
        usernameStatus({ status: "invalid", message });
    }
    else if (response.status == 409) usernameStatus({ status: "unavailable" });
    else usernameStatus({});
}, 1000);

const debounced_checkEmailAvailability = debounce(async () => {
    emailStatus({});
    const response = await fetch(`/check/email-availability?email=${document.getElementById("signup-email").value}`);
    if (response.status == 200) emailStatus({ status: "ok" });
    else if (response.status == 400) {
        const json = await response.json();
        const message = json["error"];
        emailStatus({ status: "invalid", message });
    }
    else if (response.status == 409) emailStatus({ status: "unavailable" });
    else emailStatus({});
}, 1000);

function checkPasswordStrength() {
    const password = document.getElementById("signup-password").value;
    const { valid, error } = validate_password(password);
    passwordStatus({
        status: valid ? "ok" : "weak",
        message: error
    });
}

/**
 * Handles the signup form submission.
 *
 * @param {SubmitEvent} event - The event object from the form submission.
 */
async function signup(event) {
    event.preventDefault();

    const response = await fetch("/signup", {
        method: "POST",
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            "username": document.getElementById("signup-username").value,
            "password": document.getElementById("signup-password").value,
            "email": document.getElementById("signup-email").value,
        })
    });

    if (response.ok) {
        alert("signup successful!");
        hooks.redirect("/login");
    }
    else alert(JSON.stringify(await response.json()));
}

hooks.onMount(async () => {
    window.signup = signup;
    window.debounced_checkUsernameAvailability = debounced_checkUsernameAvailability;
    window.checkPasswordStrength = checkPasswordStrength;
    window.debounced_checkEmailAvailability = debounced_checkEmailAvailability;
    await initWasm();
});
hooks.onUnmount(() => {
    delete window.signup;
    delete window.debounced_checkUsernameAvailability;
    delete window.checkPasswordStrength;
    delete window.debounced_checkEmailAvailability;
});
hooks.ready();
