const PRIMARY_NAV = document.querySelector(".primary-navigation")
const EMAIL = document.querySelector(".mobile-nav-toggle")

EMAIL.addEventListener("click", () => {
    const VISIBILITY = PRIMARY_NAV.getAttribute("data-visible") === "false"
    if (VISIBILITY) {
        PRIMARY_NAV.setAttribute("data-visible", true)
        EMAIL.setAttribute("aria-expanded", true)
    } else {
        PRIMARY_NAV.setAttribute("data-visible", false)
        EMAIL.setAttribute("aria-expanded", false)
    }
})
