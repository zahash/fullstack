import signal, { merge } from "../../lib/signal.mjs";
import debounce from "../../debounce.mjs";

const ele_username = document.getElementById("signup-username");
const ele_password = document.getElementById("signup-password");
const ele_email = document.getElementById("signup-email");
const ele_signupBtn = document.getElementById("signup-btn");

const ele_msg_username = document.getElementById("signup-msg-username");
const ele_msg_passwordLength = document.getElementById("signup-msg-password-length");
const ele_msg_passwordSpecial = document.getElementById("signup-msg-password-special");
const ele_msg_passwordDigit = document.getElementById("signup-msg-password-digit");
const ele_msg_passwordLower = document.getElementById("signup-msg-password-lower");
const ele_msg_passwordUpper = document.getElementById("signup-msg-password-upper");
const ele_msg_email = document.getElementById("signup-msg-email");

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
    ele_msg_passwordLength.style.display = val.length ? "none" : "block";
    ele_msg_passwordSpecial.style.display = val.special ? "none" : "block";
    ele_msg_passwordDigit.style.display = val.digit ? "none" : "block";
    ele_msg_passwordLower.style.display = val.lower ? "none" : "block";
    ele_msg_passwordUpper.style.display = val.upper ? "none" : "block";
});
emailStatus.effect(({ status, message }) => {
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
canSignup.effect(val => ele_signupBtn.disabled = !val);

const debounced_checkUsernameAvailability = debounce(async () => {
    usernameStatus({});
    const response = await fetch(`/check/username-availability?username=${ele_username.value}`);
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
    const response = await fetch(`/check/email-availability?email=${ele_email.value}`);
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
    const password = ele_password.value;

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
            "username": ele_username.value,
            "password": ele_password.value,
            "email": ele_email.value,
        })
    });

    if (response.ok) alert("signup successful!");
    else alert(JSON.stringify(await response.json()));
}

document.getElementById("signup-form").addEventListener("submit", signup);
ele_username.addEventListener("input", debounced_checkUsernameAvailability);
ele_password.addEventListener("input", checkPasswordStrength);
ele_email.addEventListener("input", debounced_checkEmailAvailability);
