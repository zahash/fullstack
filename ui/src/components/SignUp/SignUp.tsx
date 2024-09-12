import { Component, createEffect, createSignal, onMount } from 'solid-js';

import styles from './Signup.module.css';

const re_special = /[!@#$%^&*()_+\-=\[\]{};':"\\|,\.<>\/?]/;
const re_digit = /\d/;
const re_lowercase = /[a-z]/;
const re_uppercase = /[A-Z]/;

const taken_usernames = [
    "zahash",
    "maus",
    "rat",
    "cat"
];

function debounce<F extends (...args: any[]) => any>(fn: F, delay: number): (...args: Parameters<F>) => void {
    let timeout: NodeJS.Timeout | undefined;
    return (...args: Parameters<F>) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn(...args), delay);
    };
}

const SignUp: Component = () => {
    let usernameRef: HTMLInputElement;
    let passwordRef: HTMLInputElement;

    let [usernameAvailable, set_usernameAvailable] = createSignal<boolean | undefined>(undefined);
    let [passwordHasSpecial, set_passwordHasSpecial] = createSignal(false);
    let [passwordHasDigit, set_passwordHasDigit] = createSignal(false);
    let [passwordHasLowercase, set_passwordHasLowercase] = createSignal(false);
    let [passwordHasUppercase, set_passwordHasUppercase] = createSignal(false);

    let canSubmit = () =>
        usernameAvailable() === true &&
        passwordHasSpecial() &&
        passwordHasDigit() &&
        passwordHasLowercase() &&
        passwordHasUppercase();

    const debounced_checkUsernameAvailability = debounce(() => {
        const username = usernameRef.value;
        const available = !taken_usernames.includes(username);
        set_usernameAvailable(available);
    }, 1000);

    const checkPasswordStrength = () => {
        const password = passwordRef.value;

        set_passwordHasSpecial(re_special.test(password));
        set_passwordHasDigit(re_digit.test(password));
        set_passwordHasLowercase(re_lowercase.test(password));
        set_passwordHasUppercase(re_uppercase.test(password));
    }

    const onsubmit = (e: SubmitEvent) => {
        e.preventDefault();

        console.log(usernameRef.value);
        console.log(passwordRef.value);
    }

    return (
        <form onsubmit={onsubmit} class={styles.SignUp}>
            <input ref={ele => usernameRef = ele} type='text' placeholder='username' required
                oninput={() => { set_usernameAvailable(undefined); debounced_checkUsernameAvailability(); }} />
            <p style={{ display: usernameAvailable() === false ? "block" : "none" }}>username taken</p>

            <input ref={ele => passwordRef = ele} oninput={checkPasswordStrength} type='password' placeholder='password' required />
            <p style={{ display: passwordHasSpecial() ? "none" : "block" }}>password must have atleast one special character</p>
            <p style={{ display: passwordHasDigit() ? "none" : "block" }}>password must have atleast one digit</p>
            <p style={{ display: passwordHasLowercase() ? "none" : "block" }}>password must have atleast one lowercase letter</p>
            <p style={{ display: passwordHasUppercase() ? "none" : "block" }}>password must have atleast one uppercase letter</p>

            <button type='submit' disabled={!canSubmit()}>sign up</button>
        </form>
    );
};

export default SignUp;
