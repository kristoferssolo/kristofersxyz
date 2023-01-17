const PRIMARY_NAV = document.querySelector(".primary-navigation")
const NAV_TOGGLE = document.querySelector(".mobile-nav-toggle")

NAV_TOGGLE.addEventListener("click", () => {
    const visibility = PRIMARY_NAV.getAttribute("data-visible") === "false"
    if (visibility) {
        PRIMARY_NAV.setAttribute("data-visible", true)
        NAV_TOGGLE.setAttribute("aria-expanded", true)
    } else {
        PRIMARY_NAV.setAttribute("data-visible", false)
        NAV_TOGGLE.setAttribute("aria-expanded", false)
    }
})
