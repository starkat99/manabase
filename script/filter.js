// Can't use JQuery, isn't loaded yet
function addCSSRule(rule) {
    var style = document.createElement("style");
    style.setAttribute("type", "text/css");
    style.innerText = rule;
    document.head.appendChild(style);
}

const INITIALLY_HIDDEN_FILTERS = ["silver-border"];
const INITIALLY_SHOWN_FILTERS = ["lands", "rocks", "dorks", "ramp"];

var params = new URLSearchParams(window.location.search);
var show_filters = params.getAll("show");
var hide_filters = params.getAll("hide");

var cookie_params = Cookies.getJSON("filter");
if (cookie_params !== undefined) {
    show_filters = show_filters.concat(cookie_params["show"]);
    hide_filters = hide_filters.concat(cookie_params["hide"]);
}

function updateFilters() {
    document.querySelectorAll("head style").forEach(element => {
        element.remove();
    });
    INITIALLY_HIDDEN_FILTERS.concat(hide_filters).forEach(filter => {
        if (!show_filters.includes(filter)) {
            addCSSRule(".mtg-card[data-mtg-" + filter + "] { display: none !important; }");
        }
    });
}
updateFilters();