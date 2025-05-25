/**
 * Creates a reactive signal that can store a value and notify subscribers when updated.
 *
 * @template T
 * @param {T} initialValue - The initial value of the signal.
 * @returns {function(T=): T} A getter/setter function for the signal.
 *   - Calling the getter/setter function without arguments returns the current value.
 *   - Calling the getter/setter function with a new value updates it and notifies subscribers.
 *
 * @example
 * const count = signal(0);
 *
 * console.log(count()); // get current value
 * > 0
 *
 * count.effect(value => { // subscribe to changes
 *   console.log("Count changed to:", value);
 * });
 *
 * count(1); // Update value
 * > "Count changed to: 1"
 *
 * const doubleCount = count.derive(n => n * 2); // derived signal
 * doubleCount.effect(n => {
 *   console.log("Double count:", n);
 * });
 *
 * count(3);
 * > "Count changed to: 3"
 * > "Double count: 6"
 */
export default function signal(initialValue) {
    let value = initialValue;
    let subscribers = [];

    /**
     * Getter/setter function for the signal.
     * - If called without arguments, returns the current value.
     * - If called with a new value, updates it and notifies subscribers.
     *
     * @param {T} [newValue] - The new value to set.
     * @returns {T} The current value after potential update.
     */
    function _signal(newValue) {
        if (arguments.length === 0) return value; // Getter

        if (value !== newValue) {
            value = newValue;
            subscribers.forEach(fn => fn(value));
        }
    }

    /**
     * Subscribes a function to be called when the signal updates.
     *
     * @param {(value: T) => void} fn - The function to run on updates.
     */
    _signal.effect = (fn) => subscribers.push(fn);

    /**
     * Creates a derived signal that updates whenever the original signal changes.
     *
     * @param {(value: T) => U} transform - The transformation function.
     * @returns {function(U=): U} A new reactive signal derived from the original.
     * @template U
     */
    _signal.derive = (transform) => {
        let derived = signal(transform(value));
        _signal.effect(val => derived(transform(val)));
        return derived;
    }

    return _signal;
}

/**
 * Merges multiple signals into a single signal, where the value is an object containing the values
 * of the individual signals. Whenever any signal updates, the merged signal will also update.
 *
 * The keys in the returned object will correspond to the signal names (as provided in the input object).
 *
 * @param {Object} signals - An object where each property is a signal, and the property name is used
 *                            as the key in the merged signal's value.
 * @returns {function} A new reactive signal holding an object with the individual signal's values.
 *   - The returned signal's value is an object where each key corresponds to the original signal name.
 *
 * @example
 * const state = signal({ foo: 'bar', 1: 2 });
 * const state2 = state.derive(obj => obj[1] + 1);
 * const state3 = signal([1, 2, 3]);
 * const mergedState = merge({ state, state2, "someOtherName": state3 });
 */
export function merge(signals) {
    // Initialize mergedSignal with the initial values of the signals
    const mergedSignal = signal(Object.fromEntries(
        Object.entries(signals).map(([name, signalFn]) => [name, signalFn()])
    ));

    // Add effects to update the merged signal when any of the individual signals change
    Object.entries(signals).forEach(([name, signalFn]) => {
        signalFn.effect(value => {
            const currentValues = { ...mergedSignal() };
            currentValues[name] = value;
            mergedSignal(currentValues);
        });
    });

    return mergedSignal;
}
