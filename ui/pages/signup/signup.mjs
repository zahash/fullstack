import signal, { merge } from "../../lib/signal.mjs";
import debounce from "../../debounce.mjs";
import { hooks } from "../../app.mjs";

const re_special = /[!@#$%^&*()_+\-=\[\]{};':"\\|,\.<>\/?]/;
const re_digit = /\d/;
const re_lowercase = /[a-z]/;
const re_uppercase = /[A-Z]/;

let usernameStatus = signal({ status: undefined, message: undefined });
let passwordStatus = signal({
    length: false,
    special: false,
    digit: false,
    upper: false,
    lower: false
});
let emailStatus = signal({ status: undefined, message: undefined });

let canSignup = merge({ usernameStatus, passwordStatus, emailStatus }).derive(obj =>
    obj.usernameStatus.status === "ok" &&
    obj.passwordStatus.length &&
    obj.passwordStatus.special &&
    obj.passwordStatus.digit &&
    obj.passwordStatus.upper &&
    obj.passwordStatus.lower &&
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
passwordStatus.effect(val => {
    document.getElementById("signup-msg-password-length").style.display = val.length ? "none" : "block";
    document.getElementById("signup-msg-password-special").style.display = val.special ? "none" : "block";
    document.getElementById("signup-msg-password-digit").style.display = val.digit ? "none" : "block";
    document.getElementById("signup-msg-password-lower").style.display = val.lower ? "none" : "block";
    document.getElementById("signup-msg-password-upper").style.display = val.upper ? "none" : "block";
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

// TODO: can this function be shared between frontend and backend as a wasm module?
function checkPasswordStrength() {
    const password = document.getElementById("signup-password").value;

    passwordStatus({
        length: password.length >= 8,
        special: re_special.test(password),
        digit: re_digit.test(password),
        lower: re_lowercase.test(password),
        upper: re_uppercase.test(password)
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

    if (response.ok) alert("signup successful!");
    else alert(JSON.stringify(await response.json()));
}

window.signup = signup;
window.debounced_checkUsernameAvailability = debounced_checkUsernameAvailability;
window.checkPasswordStrength = checkPasswordStrength;
window.debounced_checkEmailAvailability = debounced_checkEmailAvailability;

hooks.onMount(() => console.log("HOOKS SIGNUP MOUNT"));
hooks.onUnmount(() => console.log("HOOKS SIGNUP UN-MOUNT"));
hooks.ready();
