console.log(document.location.href);
for (const el of document.querySelectorAll("a")) {
    console.log(el.href);
    // Remove trailing /
    el.href = el.origin + import.meta.env.BASE_URL.slice(0, -1) + el.pathname + el.search;
    console.log(el.href);
    if (el.href === document.location.href) {
        el.classList.add("current");
    }
}