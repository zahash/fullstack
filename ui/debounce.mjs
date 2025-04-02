/**
 * Creates a debounced version of a function that delays its execution.
 *
 * @param {Function} fn - The function to debounce.
 * @param {number} delay - The delay in milliseconds before executing the function.
 * @returns {Function} - A debounced function.
 */
export default function debounce(fn, delayMs) {
    let timeout;
    return (...args) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn(...args), delayMs);
    };
}
