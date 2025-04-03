async function fetchPrivate() {
    const response = await fetch("/private");
    if (response.ok) {
        console.log(await response.text());
    } else {
        console.log(await response.json());
    }
}
