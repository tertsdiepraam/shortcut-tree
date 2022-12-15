console.log(document.location.href);
for (const el of document.querySelectorAll("a")) {
    console.log(el.href);
    if (el.href === document.location.href) {
        el.classList.add("current");
    }
}