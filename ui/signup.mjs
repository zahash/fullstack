import debounce from "./debounce.js"

let usernameEle = document.getElementById("signup-username");
let passwordEle = document.getElementById("signup-password");

const re_special = /[!@#$%^&*()_+\-=\[\]{};':"\\|,\.<>\/?]/;
const re_digit = /\d/;
const re_lowercase = /[a-z]/;
const re_uppercase = /[A-Z]/;

let usernameStatus = undefined;
let passwordHasSpecial = false;
let passwordHasDigit = false;
let passwordHasLowercase = false;
let passwordHasUppercase = false;

const debounced_checkUsernameAvailability = debounce(async () => {
    const response = await fetch("/check-username-availability", {
        method: "POST",
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            "username": usernameEle.value,
        })
    });

    if (response.status == 200) usernameStatus = "ok";
    else if (response.status == 400) usernameStatus = "invalid";
    else if (response.status == 409) usernameStatus = "unavailable";
    else usernameStatus = undefined;
}, 1000);

function checkPasswordStrength() {
    const password = passwordEle.value;

    passwordHasSpecial = re_special.test(password);
    passwordHasDigit = re_digit.test(password);
    passwordHasLowercase = re_lowercase.test(password);
    passwordHasUppercase = re_uppercase.test(password);
}

function canSignup() {
    return usernameStatus == "ok" &&
        passwordHasSpecial &&
        passwordHasDigit &&
        passwordHasLowercase &&
        passwordHasUppercase;
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
            "username": usernameEle.value,
            "password": passwordEle.value,
        })
    });

    if (!response.ok) {
        const err = await response.json();
        console.log(err);
    }
}
