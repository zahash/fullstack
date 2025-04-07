async function fetchPrivate(event) {
    const response = await fetch("/private");
    if (response.ok) {
        event.target.outerHTML = await response.text();
    } else {
        console.log(await response.json());
    }
}
