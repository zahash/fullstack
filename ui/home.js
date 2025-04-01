async function private() {
    const response = await fetch("/private", { credentials: 'include' });
    if (response.ok) {
        console.log(response.body);
    } else {
        console.log(await response.json());
    }
}
